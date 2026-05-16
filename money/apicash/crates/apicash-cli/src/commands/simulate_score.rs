//! User Score: prioriza `POST /risk/score` na API core (fatores detalhados), depois admin, depois fallback local.

use anyhow::{Context, Result};
use serde_json::json;
use uuid::Uuid;

fn core_base() -> String {
    std::env::var("APICASH_HTTP_BASE").unwrap_or_else(|_| "http://127.0.0.1:3000".into())
}

fn bearer() -> Option<String> {
    std::env::var("APICASH_BEARER_TOKEN")
        .ok()
        .filter(|s| !s.is_empty())
}

fn admin_base() -> String {
    std::env::var("APICASH_ADMIN_HTTP_BASE").unwrap_or_else(|_| "http://127.0.0.1:3001".into())
}

fn admin_key() -> Option<String> {
    std::env::var("APICASH_ADMIN_API_KEY")
        .ok()
        .filter(|s| !s.is_empty())
}

fn placeholder_score_from_cpf(cpf: &str) -> u32 {
    let digits: String = cpf.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return 0;
    }
    let sum: u64 = digits.bytes().map(u64::from).sum();
    ((sum * 7919) % 1001) as u32
}

fn print_factor_breakdown_local(score: u32, cpf_tail: &str) {
    println!("Modo *offline* (sem API): fatores ilustrativos derivados do score {score} (CPF ***{cpf_tail})");
    println!();
    println!(
        "  • SEFAZ / situação cadastral  → peso estimado: +{}",
        (score / 3).min(350)
    );
    println!(
        "  • Histórico de disputas      → peso estimado: -{}",
        (1000u32.saturating_sub(score)) / 50
    );
    println!(
        "  • Perfil social (placeholder)→ peso estimado: +{}",
        (score / 5).min(120)
    );
    println!(
        "  • Ajustes diversos           → peso estimado: +{}",
        (score % 40) as i32
    );
    println!();
    println!(
        "Use `POST /risk/score` na API core para fatores reais (`simulate-score` com API ligada)."
    );
}

/// Formata um fator JSON (tag + campos) para linha legível.
fn format_risk_factor(v: &serde_json::Value) -> String {
    let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("unknown");
    match kind {
        "sefaz_status" => {
            let w = v.get("weight").and_then(|x| x.as_i64()).unwrap_or(0);
            format!("SEFAZ / cadastro regular — peso {w:+}")
        }
        "social_account_age" => {
            let plat = v.get("platform").and_then(|x| x.as_str()).unwrap_or("?");
            let months = v.get("months").and_then(|x| x.as_u64()).unwrap_or(0);
            let w = v.get("weight").and_then(|x| x.as_i64()).unwrap_or(0);
            format!("Rede social ({plat}): idade ~{months} m — peso {w:+}")
        }
        "transaction_history" => {
            let w = v.get("weight").and_then(|x| x.as_i64()).unwrap_or(0);
            format!("Histórico de transações — peso {w:+}")
        }
        "dispute_history" => {
            let n = v.get("open_disputes").and_then(|x| x.as_u64()).unwrap_or(0);
            let w = v.get("weight").and_then(|x| x.as_i64()).unwrap_or(0);
            format!("Disputas em aberto: {n} — peso {w:+}")
        }
        "other" => {
            let code = v.get("code").and_then(|x| x.as_str()).unwrap_or("");
            let w = v.get("weight").and_then(|x| x.as_i64()).unwrap_or(0);
            format!("Outro ({code}) — peso {w:+}")
        }
        _ => v.to_string(),
    }
}

pub async fn run(cpf: &str) -> Result<()> {
    let digits: String = cpf.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 11 {
        anyhow::bail!("CPF deve ter 11 dígitos");
    }
    let tail = &digits[7..];

    let client = reqwest::Client::new();
    let user_id = Uuid::new_v4();

    // 1) API core — fatores completos
    let url = format!("{}/risk/score", core_base().trim_end_matches('/'));
    let mut req = client.post(&url).json(&json!({
        "user_id": user_id,
        "cpf": &digits,
        "social_links": [],
    }));
    if let Some(ref t) = bearer() {
        req = req.header("Authorization", format!("Bearer {t}"));
    }

    tracing::info!(%url, "simulate-score: tentando POST /risk/score na API core");
    match req.send().await {
        Ok(res) if res.status().is_success() => {
            let txt = res.text().await.context("body /risk/score")?;
            let v: serde_json::Value = serde_json::from_str(&txt).context("parse /risk/score")?;
            let score = v.get("score").and_then(|s| s.as_u64()).unwrap_or(0);
            let level = v.get("risk_level").and_then(|s| s.as_str()).unwrap_or("?");
            let decision = v.get("decision").and_then(|s| s.as_str()).unwrap_or("?");
            println!("User Score (API core) para CPF ***{tail}: {score}");
            println!("Nível de risco: {level} | decisão on-ramp: {decision}");
            println!();
            println!("Fatores que compõem o score:");
            if let Some(arr) = v.get("factors").and_then(|f| f.as_array()) {
                if arr.is_empty() {
                    println!("  (nenhum fator retornado)");
                } else {
                    for (i, fac) in arr.iter().enumerate() {
                        println!("  {}. {}", i + 1, format_risk_factor(fac));
                    }
                }
            } else {
                println!("  (campo factors ausente)");
            }
            return Ok(());
        }
        Ok(res) => {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            tracing::warn!(%status, %body, "POST /risk/score não OK — tentando admin ou fallback");
        }
        Err(e) => {
            tracing::warn!(error = %e, "POST /risk/score indisponível — tentando admin ou fallback");
        }
    }

    // 2) Admin API (lista sem fatores — aviso)
    if let Some(key) = admin_key() {
        let url = format!("{}/admin/users/score", admin_base().trim_end_matches('/'));
        let res = client
            .get(&url)
            .header("x-api-key", &key)
            .send()
            .await
            .context("GET /admin/users/score")?;
        if res.status().is_success() {
            let body = res.text().await?;
            println!("Resposta admin (sem fatores detalhados por utilizador):");
            println!("{body}");
            println!();
            print_factor_breakdown_local(placeholder_score_from_cpf(&digits), tail);
            return Ok(());
        }
        tracing::warn!("admin API não OK");
    } else {
        tracing::info!("APICASH_ADMIN_API_KEY não definido");
    }

    let score = placeholder_score_from_cpf(&digits);
    println!("Score simulado (dev) para CPF ***{tail}: {score}");
    println!();
    print_factor_breakdown_local(score, tail);
    Ok(())
}
