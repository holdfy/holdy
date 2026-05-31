//! Consulta cadastral via portal NFS-e Nacional (dados da Receita Federal).
//!
//! Requires `NFSE_INSCRICAO` + `NFSE_SENHA` in the environment.
//! All errors are soft: returns `None` rather than blocking the order flow.

use chrono::Utc;
use reqwest::Client;

const BASE: &str = "https://www.nfse.gov.br";

/// Dados retornados pela consulta ao portal (nome / situação cadastral quando disponível).
#[derive(Debug, Clone)]
pub struct PersonLookup {
    pub document_digits: String,
    pub document_label: &'static str,
    pub name: Option<String>,
    pub situation: Option<String>,
}

impl PersonLookup {
    /// Mensagem para WhatsApp após consulta (papel: "vendedor", "comprador", etc.).
    #[must_use]
    pub fn whatsapp_summary(&self, role: &str) -> String {
        let doc = format_document_display(&self.document_digits);
        let mut lines = vec![format!("*{}* — {doc}", role)];
        if let Some(ref name) = self.name {
            lines.push(format!("Nome: *{name}*"));
        }
        if let Some(ref sit) = self.situation {
            lines.push(format!("Situação cadastral: *{sit}*"));
        }
        if self.name.is_none() && self.situation.is_none() {
            lines.push(
                "Documento aceito. Não foi possível obter nome na Receita agora.".into(),
            );
        } else {
            lines.push("_Fonte: consulta NFS-e / Receita Federal_".into());
        }
        lines.join("\n")
    }
}

/// Motivo quando o nome não veio da Receita (para mensagem no WhatsApp).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupStatus {
    /// `NFSE_INSCRICAO` / `NFSE_SENHA` ausentes no ambiente do apicash-whatsapp.
    NotConfigured,
    /// Login ou API do portal falhou.
    PortalError,
    /// Portal respondeu, mas sem campo de nome reconhecido.
    NameNotInResponse,
    /// Nome (e/ou situação) obtidos.
    Ok,
}

/// Consulta CPF/CNPJ no portal NFS-e (Receita).
pub async fn lookup_person(document: &str) -> (Option<PersonLookup>, LookupStatus) {
    let inscricao = match std::env::var("NFSE_INSCRICAO").ok().filter(|s| !s.trim().is_empty()) {
        Some(v) => v,
        None => {
            tracing::warn!("nfse_client: NFSE_INSCRICAO não configurado — consulta Receita ignorada");
            return (None, LookupStatus::NotConfigured);
        }
    };
    let senha = match std::env::var("NFSE_SENHA").ok().filter(|s| !s.trim().is_empty()) {
        Some(v) => v,
        None => {
            tracing::warn!("nfse_client: NFSE_SENHA não configurado — consulta Receita ignorada");
            return (None, LookupStatus::NotConfigured);
        }
    };

    let digits: String = document.chars().filter(|c| c.is_ascii_digit()).collect();
    let document_label = if digits.len() == 11 {
        "CPF"
    } else if digits.len() == 14 {
        "CNPJ"
    } else {
        return (None, LookupStatus::PortalError);
    };

    let client = match Client::builder()
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "nfse_client: http client build failed");
            return (None, LookupStatus::PortalError);
        }
    };

    if let Err(e) = login(&client, &inscricao, &senha).await {
        tracing::warn!(error = %e, "nfse_client: login falhou");
        return (None, LookupStatus::PortalError);
    }

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let json = if digits.len() == 11 {
        fetch_cpf_json(&client, &digits, &today).await
    } else {
        fetch_cnpj_json(&client, &digits, &today).await
    };
    let json = match json {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, doc_len = digits.len(), "nfse_client: API consulta falhou");
            return (None, LookupStatus::PortalError);
        }
    };

    let name = extract_name_from_value(&json);
    let situation = extract_situation_from_value(&json);
    let person = PersonLookup {
        document_digits: digits,
        document_label,
        name: name.clone(),
        situation: situation.clone(),
    };
    if name.is_some() || situation.is_some() {
        tracing::info!(
            doc_len = person.document_digits.len(),
            name = name.as_deref().unwrap_or("?"),
            "nfse_client: consulta Receita ok"
        );
        (Some(person), LookupStatus::Ok)
    } else {
        tracing::warn!(
            doc_len = person.document_digits.len(),
            body = %json,
            "nfse_client: resposta sem nome reconhecido — body completo acima para diagnóstico"
        );
        (Some(person), LookupStatus::NameNotInResponse)
    }
}

/// Nome completo (compatibilidade com chamadas antigas).
pub async fn lookup_name(document: &str) -> Option<String> {
    lookup_person(document)
        .await
        .0
        .and_then(|p| p.name)
}

fn format_document_display(digits: &str) -> String {
    match digits.len() {
        11 => format!(
            "{}.{}.{}-{}",
            &digits[0..3],
            &digits[3..6],
            &digits[6..9],
            &digits[9..11]
        ),
        14 => format!(
            "{}.{}.{}/{}-{}",
            &digits[0..2],
            &digits[2..5],
            &digits[5..8],
            &digits[8..12],
            &digits[12..14]
        ),
        _ => digits.to_string(),
    }
}

async fn login(client: &Client, inscricao: &str, senha: &str) -> Result<(), String> {
    // Step 1: GET login page to extract anti-forgery token.
    let page = client
        .get(format!("{BASE}/EmissorNacional/Login"))
        .header("User-Agent", "Mozilla/5.0 (compatible; HoldFy/1.0)")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let csrf = extract_csrf_token(&page)
        .ok_or_else(|| "CSRF token not found in login page".to_string())?;

    // Step 2: POST credentials.
    let resp = client
        .post(format!("{BASE}/EmissorNacional/Login"))
        .header("User-Agent", "Mozilla/5.0 (compatible; HoldFy/1.0)")
        .header("Referer", format!("{BASE}/EmissorNacional/Login"))
        .form(&[
            ("Inscricao", inscricao),
            ("Senha", senha),
            ("__RequestVerificationToken", &csrf),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    let final_url = resp.url().clone();
    let body = resp.text().await.unwrap_or_default();

    if !status.is_success() && status.as_u16() != 302 {
        return Err(format!("login HTTP {status} url={final_url}"));
    }
    if final_url.path().contains("/Login") {
        // Log primeiras linhas do body para diagnóstico (sem expor senha)
        let preview: String = body.lines().take(5).collect::<Vec<_>>().join(" ");
        tracing::warn!(
            url = %final_url,
            body_preview = %preview,
            "nfse_client: login rejeitado — credenciais erradas ou portal bloqueou"
        );
        return Err("login rejeitado (credenciais ou portal)".into());
    }
    tracing::debug!(url = %final_url, "nfse_client: login ok");
    Ok(())
}

const NFSE_REFERER: &str = "https://www.nfse.gov.br/EmissorNacional/";

async fn fetch_cpf_json(client: &Client, cpf: &str, date: &str) -> Result<serde_json::Value, String> {
    let url = format!(
        "{BASE}/emissornacional/api/EmissaoDPS/RecuperarInfoInscricao/{cpf}?data={date}"
    );
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (compatible; HoldFy/1.0)")
        .header("Accept", "application/json")
        .header("Referer", NFSE_REFERER)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("CPF lookup HTTP {status}: {body}"));
    }
    resp.json().await.map_err(|e| e.to_string())
}

async fn fetch_cnpj_json(client: &Client, cnpj: &str, date: &str) -> Result<serde_json::Value, String> {
    let url = format!(
        "{BASE}/emissornacional/api/EmissaoDPS/RecuperarInfoPessoaJuridicaTomador/{cnpj}?data={date}"
    );
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (compatible; HoldFy/1.0)")
        .header("Accept", "application/json")
        .header("Referer", NFSE_REFERER)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("CNPJ lookup HTTP {status}: {body}"));
    }
    resp.json().await.map_err(|e| e.to_string())
}

fn extract_csrf_token(html: &str) -> Option<String> {
    // <input name="__RequestVerificationToken" type="hidden" value="..." />
    let marker = "name=\"__RequestVerificationToken\"";
    let pos = html.find(marker)?;
    let after = &html[pos..];
    let val_start = after.find("value=\"")?;
    let after_val = &after[val_start + 7..];
    let val_end = after_val.find('"')?;
    Some(after_val[..val_end].to_string())
}

fn extract_name_from_value(v: &serde_json::Value) -> Option<String> {
    const FIELDS: &[&str] = &[
        "nomeRazaoSocial",
        "razaoSocial",
        "nome",
        "nomeContribuinte",
        "nomeEmpresarial",
        "nomePessoa",
        "nomeCompleto",
        "nomeSocial",
    ];
    extract_field_from_value(v, FIELDS, title_case)
}

fn extract_situation_from_value(v: &serde_json::Value) -> Option<String> {
    const FIELDS: &[&str] = &[
        "situacaoCadastral",
        "situacao",
        "descricaoSituacaoCadastral",
        "codigoSituacaoCadastral",
    ];
    extract_field_from_value(v, FIELDS, |s| s.to_string())
}

fn field_key_matches(key: &str, candidates: &[&str]) -> bool {
    let k = key.to_ascii_lowercase();
    candidates
        .iter()
        .any(|c| k == c.to_ascii_lowercase())
}

fn extract_field_from_value(
    v: &serde_json::Value,
    fields: &[&str],
    transform: fn(&str) -> String,
) -> Option<String> {
    if let serde_json::Value::Object(obj) = v {
        for (key, val) in obj {
            if field_key_matches(key, fields) {
                if let Some(s) = val.as_str() {
                    let s = s.trim();
                    if !s.is_empty() {
                        return Some(transform(s));
                    }
                }
            }
        }
    }
    match v {
        serde_json::Value::Object(obj) => {
            for child in obj.values() {
                if let Some(found) = extract_field_from_value(child, fields, transform) {
                    return Some(found);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for child in items {
                if let Some(found) = extract_field_from_value(child, fields, transform) {
                    return Some(found);
                }
            }
        }
        _ => {}
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Corpo real retornado por curl (ideia.txt) para CPF 86481096987 em 2026-05-31.
    const SAMPLE_CPF_JSON: &str =
        r#"{"inscricao":"86481096987","nomerazaosocial":"SILVIO CEZAR SACZUCK","codigopais":0}"#;

    #[test]
    fn extract_name_from_nfse_lowercase_keys() {
        let v: serde_json::Value = serde_json::from_str(SAMPLE_CPF_JSON).unwrap();
        let name = extract_name_from_value(&v).expect("nomerazaosocial must map to nome");
        assert_eq!(name, "Silvio Cezar Saczuck");
    }

    #[test]
    fn field_key_matches_case_insensitive() {
        assert!(field_key_matches("nomerazaosocial", &["nomeRazaoSocial"]));
        assert!(field_key_matches("nomeRazaoSocial", &["nomeRazaoSocial"]));
    }
}

/// Convert ALL-CAPS government name to Title Case.
fn title_case(s: &str) -> String {
    let lower = ["da", "de", "do", "das", "dos", "e", "em", "na", "no", "nas", "nos"];
    s.split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            let w = word.to_lowercase();
            if i > 0 && lower.contains(&w.as_str()) {
                w
            } else {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
