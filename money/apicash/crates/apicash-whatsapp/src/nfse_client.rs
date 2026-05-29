//! Consulta de nome via portal NFS-e (gov.br).
//!
//! Requires NFSE_INSCRICAO + NFSE_SENHA in the environment.
//! All errors are soft: returns `None` rather than blocking the order flow.

use chrono::Utc;
use reqwest::Client;

const BASE: &str = "https://www.nfse.gov.br";

/// Try to resolve the buyer's full name via the NFS-e government portal.
///
/// `document` must be either 11 digits (CPF) or 14 digits (CNPJ).
/// Returns `None` when credentials are absent, the portal is unreachable, or the document is unknown.
pub async fn lookup_name(document: &str) -> Option<String> {
    let inscricao = std::env::var("NFSE_INSCRICAO").ok().filter(|s| !s.trim().is_empty())?;
    let senha = std::env::var("NFSE_SENHA").ok().filter(|s| !s.trim().is_empty())?;

    let digits: String = document.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 11 && digits.len() != 14 {
        return None;
    }

    let client = Client::builder()
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .ok()?;

    if let Err(e) = login(&client, &inscricao, &senha).await {
        tracing::warn!(error = %e, "nfse_client: login falhou — nome do comprador não disponível");
        return None;
    }

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let result = if digits.len() == 11 {
        fetch_cpf_name(&client, &digits, &today).await
    } else {
        fetch_cnpj_name(&client, &digits, &today).await
    };

    match result {
        Ok(name) => {
            tracing::info!(doc_len = digits.len(), name = %name, "nfse_client: nome obtido");
            Some(name)
        }
        Err(e) => {
            tracing::warn!(error = %e, doc_len = digits.len(), "nfse_client: consulta falhou");
            None
        }
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
            ("inscricao", inscricao),
            ("senha", senha),
            ("__RequestVerificationToken", &csrf),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() && resp.status().as_u16() != 302 {
        return Err(format!("login HTTP {}", resp.status()));
    }
    Ok(())
}

async fn fetch_cpf_name(client: &Client, cpf: &str, date: &str) -> Result<String, String> {
    let url = format!(
        "{BASE}/emissornacional/api/EmissaoDPS/RecuperarInfoInscricao/{cpf}?data={date}"
    );
    let json: serde_json::Value = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (compatible; HoldFy/1.0)")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    extract_name_from_value(&json).ok_or_else(|| format!("name field not found: {json}"))
}

async fn fetch_cnpj_name(client: &Client, cnpj: &str, date: &str) -> Result<String, String> {
    let url = format!(
        "{BASE}/emissornacional/api/EmissaoDPS/RecuperarInfoPessoaJuridicaTomador/{cnpj}?data={date}"
    );
    let json: serde_json::Value = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (compatible; HoldFy/1.0)")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    extract_name_from_value(&json).ok_or_else(|| format!("name field not found: {json}"))
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
    // Try common field names in order of likelihood.
    for field in &[
        "nomeRazaoSocial",
        "razaoSocial",
        "nome",
        "nomeContribuinte",
        "nomeEmpresarial",
        "nomePessoa",
    ] {
        if let Some(s) = v.get(field).and_then(|f| f.as_str()) {
            let s = s.trim();
            if !s.is_empty() {
                return Some(title_case(s));
            }
        }
    }
    None
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
