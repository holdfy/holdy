//! KYC: consulta CPF/CNPJ via portal NFS-e / Receita Federal.
//!
//! Cache hierarchy (Redis TTL 24h) → Postgres `nfse_document_cache` → NFS-e portal.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    Json,
};
use redis::{aio::ConnectionManager, AsyncCommands};
use reqwest::Client;
use serde::Serialize;

use crate::error::ApiError;
use crate::state::AppState;

const KYC_REDIS_KEY_PREFIX: &str = "nfse:v1:";
const KYC_REDIS_TTL_SECS: u64 = 86_400;
const BASE: &str = "https://www.nfse.gov.br";
const UA: &str = "Mozilla/5.0 (compatible; HoldFy/1.0)";
const NFSE_REFERER: &str = "https://www.nfse.gov.br/EmissorNacional/";

#[derive(Debug, Clone, Serialize)]
pub struct KycResponse {
    pub document: String,
    pub document_type: String,
    pub name: Option<String>,
    pub situation: Option<String>,
    pub source: String,
}

/// `GET /kyc/document/{document}` — requer JWT (middleware).
pub async fn lookup_document(
    State(state): State<Arc<AppState>>,
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

    // 1. Redis
    if let Some(mut conn) = state.kyc_redis.clone() {
        if let Some(resp) = redis_get(&mut conn, &digits, doc_type).await {
            tracing::info!(doc_len = digits.len(), "kyc: redis hit");
            return Ok(Json(resp));
        }
    }

    // 2. Postgres
    if let Some(pool) = &state.pool {
        if let Some(resp) = pg_get(pool, &digits, doc_type).await {
            tracing::info!(doc_len = digits.len(), "kyc: postgres hit");
            if let Some(mut conn) = state.kyc_redis.clone() {
                let r = resp.clone();
                tokio::spawn(async move { redis_set(&mut conn, &r).await });
            }
            return Ok(Json(resp));
        }
    }

    // 3. NFS-e portal
    let (name, situation) = nfse_lookup(&digits).await;
    let resp = KycResponse {
        document: digits.clone(),
        document_type: doc_type.into(),
        name,
        situation,
        source: "NFS-e / Receita Federal".into(),
    };

    // Persiste nos caches (fire-and-forget) — apenas quando há nome
    if resp.name.is_some() {
        let r = resp.clone();
        let pool_opt = state.pool.clone();
        let redis_opt = state.kyc_redis.clone();
        tokio::spawn(async move {
            if let Some(pool) = pool_opt {
                pg_set(&pool, &r).await;
            }
            if let Some(mut conn) = redis_opt {
                redis_set(&mut conn, &r).await;
            }
        });
    }

    Ok(Json(resp))
}

// ─── Cache Redis ──────────────────────────────────────────────────────────────

async fn redis_get(conn: &mut ConnectionManager, digits: &str, doc_type: &str) -> Option<KycResponse> {
    let key = format!("{KYC_REDIS_KEY_PREFIX}{digits}");
    let raw: Option<String> = conn.get(&key).await.ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw?).ok()?;
    Some(KycResponse {
        document: digits.to_string(),
        document_type: doc_type.to_string(),
        name: v.get("n").and_then(|x| x.as_str()).map(|s| s.to_string()),
        situation: v.get("s").and_then(|x| x.as_str()).map(|s| s.to_string()),
        source: "cache".into(),
    })
}

async fn redis_set(conn: &mut ConnectionManager, resp: &KycResponse) {
    let key = format!("{KYC_REDIS_KEY_PREFIX}{}", resp.document);
    let payload = serde_json::json!({ "n": resp.name, "s": resp.situation });
    let _ = conn
        .set_ex::<_, _, ()>(&key, payload.to_string(), KYC_REDIS_TTL_SECS)
        .await;
}

// ─── Cache Postgres ───────────────────────────────────────────────────────────

async fn pg_get(pool: &sqlx::PgPool, digits: &str, doc_type: &str) -> Option<KycResponse> {
    use sqlx::Row;
    let row = sqlx::query(
        "SELECT name, situation FROM nfse_document_cache WHERE document = $1 AND name IS NOT NULL",
    )
    .bind(digits)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    Some(KycResponse {
        document: digits.to_string(),
        document_type: doc_type.to_string(),
        name: row.try_get::<Option<String>, _>("name").ok().flatten(),
        situation: row.try_get::<Option<String>, _>("situation").ok().flatten(),
        source: "cache".into(),
    })
}

async fn pg_set(pool: &sqlx::PgPool, resp: &KycResponse) {
    let doc_type = if resp.document.len() == 11 { "cpf" } else { "cnpj" };
    let _ = sqlx::query(
        "INSERT INTO nfse_document_cache (document, document_type, name, situation)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (document) DO UPDATE SET
             name      = EXCLUDED.name,
             situation = COALESCE(EXCLUDED.situation, nfse_document_cache.situation),
             cached_at = NOW()",
    )
    .bind(&resp.document)
    .bind(doc_type)
    .bind(&resp.name)
    .bind(&resp.situation)
    .execute(pool)
    .await;
}

// ─── NFS-e portal ─────────────────────────────────────────────────────────────

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

    let csrf = extract_csrf(&page).ok_or_else(|| "CSRF token not found".to_string())?;

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
