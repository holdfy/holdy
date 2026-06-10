//! KYC: consulta CPF/CNPJ via portal NFS-e / Receita Federal.
//!
//! Resultados ficam em cache em memória por 24 h para evitar chamadas duplicadas
//! ao portal (ex.: comprador e vendedor com o mesmo documento).

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use axum::{extract::Path, Json};
use reqwest::Client;
use serde::Serialize;

use crate::error::ApiError;

const CACHE_TTL: Duration = Duration::from_secs(24 * 3600);
const BASE: &str = "https://www.nfse.gov.br";
const UA: &str = "Mozilla/5.0 (compatible; HoldFy/1.0)";
const NFSE_REFERER: &str = "https://www.nfse.gov.br/EmissorNacional/";

static KYC_CACHE: OnceLock<Mutex<HashMap<String, (KycResponse, Instant)>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<String, (KycResponse, Instant)>> {
    KYC_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Clone, Serialize)]
pub struct KycResponse {
    pub document: String,
    pub document_type: String,
    pub name: Option<String>,
    pub situation: Option<String>,
    pub source: String,
}

/// `GET /kyc/document/{document}` — requires JWT auth (middleware handles it).
pub async fn lookup_document(
    Path(document): Path<String>,
) -> Result<Json<KycResponse>, ApiError> {
    let digits: String = document.chars().filter(|c| c.is_ascii_digit()).collect();
    let doc_type = match digits.len() {
        11 => "CPF",
        14 => "CNPJ",
        _ => {
            return Err(ApiError::bad_request(
                "Documento inválido. Informe CPF (11 dígitos) ou CNPJ (14 dígitos).",
            ))
        }
    };

    // Serve from cache if not stale.
    {
        let lock = cache().lock().unwrap_or_else(|p| p.into_inner());
        if let Some((resp, ts)) = lock.get(&digits) {
            if ts.elapsed() < CACHE_TTL {
                return Ok(Json(resp.clone()));
            }
        }
    }

    // Consult the NFS-e portal (credentials optional: soft failure if absent).
    let (name, situation) = nfse_lookup(&digits).await;

    let resp = KycResponse {
        document: digits.clone(),
        document_type: doc_type.into(),
        name,
        situation,
        source: "NFS-e / Receita Federal".into(),
    };

    // Store in cache.
    {
        let mut lock = cache().lock().unwrap_or_else(|p| p.into_inner());
        lock.insert(digits, (resp.clone(), Instant::now()));
    }

    Ok(Json(resp))
}

// ─── Internal NFS-e helpers ──────────────────────────────────────────────────

async fn nfse_lookup(digits: &str) -> (Option<String>, Option<String>) {
    let inscricao = match std::env::var("NFSE_INSCRICAO")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        Some(v) => v,
        None => {
            tracing::debug!("kyc_handler: NFSE_INSCRICAO not set — skipping Receita lookup");
            return (None, None);
        }
    };
    let senha = match std::env::var("NFSE_SENHA")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        Some(v) => v,
        None => {
            tracing::debug!("kyc_handler: NFSE_SENHA not set — skipping Receita lookup");
            return (None, None);
        }
    };

    let client = match Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "kyc_handler: http client build failed");
            return (None, None);
        }
    };

    if let Err(e) = nfse_login(&client, &inscricao, &senha).await {
        tracing::warn!(error = %e, "kyc_handler: NFS-e login failed");
        return (None, None);
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let json = if digits.len() == 11 {
        fetch_cpf(&client, digits, &today).await
    } else {
        fetch_cnpj(&client, digits, &today).await
    };

    let json = match json {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "kyc_handler: NFS-e API call failed");
            return (None, None);
        }
    };

    let name = extract_field(
        &json,
        &[
            "nomeRazaoSocial", "razaoSocial", "nome", "nomeContribuinte",
            "nomeEmpresarial", "nomePessoa", "nomeCompleto", "nomeSocial",
        ],
        title_case,
    );
    let situation = extract_field(
        &json,
        &[
            "situacaoCadastral", "situacao", "descricaoSituacaoCadastral",
            "codigoSituacaoCadastral",
        ],
        |s| s.to_string(),
    );

    (name, situation)
}

async fn nfse_login(client: &Client, inscricao: &str, senha: &str) -> Result<(), String> {
    let page = client
        .get(format!("{BASE}/EmissorNacional/Login"))
        .header("User-Agent", UA)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let csrf = extract_csrf(&page)
        .ok_or_else(|| "CSRF token not found".to_string())?;

    let resp = client
        .post(format!("{BASE}/EmissorNacional/Login"))
        .header("User-Agent", UA)
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
    let url = resp.url().clone();
    let _ = resp.text().await;

    if !status.is_success() && status.as_u16() != 302 {
        return Err(format!("login HTTP {status}"));
    }
    if url.path().contains("/Login") {
        return Err("login rejected — wrong credentials or portal blocked".into());
    }
    Ok(())
}

async fn fetch_cpf(client: &Client, cpf: &str, date: &str) -> Result<serde_json::Value, String> {
    let url = format!("{BASE}/emissornacional/api/EmissaoDPS/RecuperarInfoInscricao/{cpf}?data={date}");
    let resp = client
        .get(&url)
        .header("User-Agent", UA)
        .header("Accept", "application/json")
        .header("Referer", NFSE_REFERER)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("CPF HTTP {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

async fn fetch_cnpj(client: &Client, cnpj: &str, date: &str) -> Result<serde_json::Value, String> {
    let url = format!("{BASE}/emissornacional/api/EmissaoDPS/RecuperarInfoPessoaJuridicaTomador/{cnpj}?data={date}");
    let resp = client
        .get(&url)
        .header("User-Agent", UA)
        .header("Accept", "application/json")
        .header("Referer", NFSE_REFERER)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("CNPJ HTTP {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

fn extract_csrf(html: &str) -> Option<String> {
    let marker = "name=\"__RequestVerificationToken\"";
    let pos = html.find(marker)?;
    let after = &html[pos..];
    let start = after.find("value=\"")? + 7;
    let end = after[start..].find('"')?;
    Some(after[start..start + end].to_string())
}

fn extract_field(
    v: &serde_json::Value,
    fields: &[&str],
    transform: fn(&str) -> String,
) -> Option<String> {
    if let serde_json::Value::Object(obj) = v {
        for (key, val) in obj {
            if fields.iter().any(|f| key.to_ascii_lowercase() == f.to_ascii_lowercase()) {
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
                if let Some(found) = extract_field(child, fields, transform) {
                    return Some(found);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for child in items {
                if let Some(found) = extract_field(child, fields, transform) {
                    return Some(found);
                }
            }
        }
        _ => {}
    }
    None
}

fn title_case(s: &str) -> String {
    let lower = ["da", "de", "do", "das", "dos", "e", "em", "na", "no", "nas", "nos"];
    s.split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            let w = word.to_lowercase();
            if i > 0 && lower.contains(&w.as_str()) {
                w
            } else {
                let mut chars = w.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
