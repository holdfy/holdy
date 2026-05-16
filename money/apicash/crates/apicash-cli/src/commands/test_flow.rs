//! Fluxo ponta a ponta contra `apicash-core`: score → pedido → PIX adicional (mock) → confirmação → yield → off-ramp.

use anyhow::{Context, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::json;
use std::time::{Duration, Instant};
use uuid::Uuid;

fn base_url() -> String {
    std::env::var("APICASH_HTTP_BASE").unwrap_or_else(|_| "http://127.0.0.1:3000".into())
}

fn bearer_env() -> Option<String> {
    std::env::var("APICASH_BEARER_TOKEN")
        .ok()
        .filter(|s| !s.is_empty())
}

async fn resolve_token(
    client: &reqwest::Client,
    user_env: &str,
    pass_env: &str,
    default_user: &str,
    default_pass: &str,
) -> Result<String> {
    if let (Ok(u), Ok(p)) = (std::env::var(user_env), std::env::var(pass_env)) {
        if let Some(t) = login(client, &u, &p).await? {
            return Ok(t);
        }
        anyhow::bail!("credenciais inválidas em {user_env}/{pass_env}");
    }

    if let Some(t) = bearer_env() {
        return Ok(t);
    }

    if let Some(t) = login(client, default_user, default_pass).await? {
        return Ok(t);
    }

    anyhow::bail!(
        "auth habilitado e nenhum token disponível. Defina APICASH_BEARER_TOKEN, ou {user_env}/{pass_env}, ou desabilite auth (APICASH_AUTH_DISABLED=1) e reinicie a API"
    );
}

async fn authed_post(
    client: &reqwest::Client,
    path: &str,
    body: &serde_json::Value,
    bearer: Option<&str>,
) -> Result<reqwest::Response, reqwest::Error> {
    let url = format!("{}{}", base_url().trim_end_matches('/'), path);
    let mut req = client.post(url).json(body);
    if let Some(t) = bearer {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    req.send().await
}

async fn login(client: &reqwest::Client, username: &str, password: &str) -> Result<Option<String>> {
    let url = format!("{}/auth/login", base_url().trim_end_matches('/'));
    let body = json!({ "username": username, "password": password });
    let res = client.post(url).json(&body).send().await?;
    let status = res.status();
    let text = res.text().await?;
    if !status.is_success() {
        tracing::warn!(%status, body = %text, "test-flow: login falhou");
        return Ok(None);
    }
    let v: serde_json::Value = serde_json::from_str(&text).context("parse login json")?;
    Ok(v.get("access_token")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string()))
}

fn user_id_from_jwt(access_token: &str) -> Result<Uuid> {
    let payload = access_token
        .split('.')
        .nth(1)
        .context("invalid JWT: missing payload")?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload.as_bytes())
        .context("base64url decode JWT payload")?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).context("parse JWT payload JSON")?;
    let sub = v
        .get("sub")
        .and_then(|x| x.as_str())
        .context("JWT payload missing sub")?;
    Uuid::parse_str(sub).context("parse JWT sub as uuid")
}

pub async fn run() -> Result<()> {
    let base = base_url();
    let client = reqwest::Client::new();
    let amount = "10.00";
    // Use a "regular" CPF + a stable social link so mock antifraude doesn't block the flow.
    let cpf = "52998224725";

    // Prefer real auth binding when possible.
    let buyer_token = resolve_token(
        &client,
        "APICASH_TEST_BUYER_USERNAME",
        "APICASH_TEST_BUYER_PASSWORD",
        "buyer",
        "buyer",
    )
    .await?;
    let seller_token = resolve_token(
        &client,
        "APICASH_TEST_SELLER_USERNAME",
        "APICASH_TEST_SELLER_PASSWORD",
        "seller",
        "seller",
    )
    .await?;

    let buyer_token = Some(buyer_token.as_str());
    let seller_token = Some(seller_token.as_str());

    let buyer = buyer_token
        .map(user_id_from_jwt)
        .transpose()
        .unwrap_or(None)
        .unwrap_or_else(Uuid::new_v4);
    let seller = seller_token
        .map(user_id_from_jwt)
        .transpose()
        .unwrap_or(None)
        .unwrap_or_else(Uuid::new_v4);

    tracing::info!(%buyer, %seller, %base, "=== APICash test-flow: início ===");

    let soroban_env = std::env::var("APICASH_SOROBAN_ENABLED")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    println!(
        "Custódia / API: {} (defina APICASH_SOROBAN_ENABLED=1 + credenciais Soroban para fluxo testnet real)",
        if soroban_env {
            "Soroban ativado (env)"
        } else {
            "mock / off-chain (padrão)"
        }
    );

    let t0 = Instant::now();
    let mut step_times: Vec<(&'static str, Duration)> = Vec::new();

    // 1) User score (antifraude) — fatores completos via API
    let t_step = Instant::now();
    let risk_body = json!({
        "user_id": buyer,
        "cpf": cpf,
        "social_links": ["https://instagram.com/old_user"],
    });
    let risk_res = authed_post(&client, "/risk/score", &risk_body, buyer_token)
        .await
        .context("POST /risk/score")?;
    let risk_status = risk_res.status();
    let risk_txt = risk_res.text().await?;
    if !risk_status.is_success() {
        anyhow::bail!("POST /risk/score failed: {risk_status} — {risk_txt}");
    }
    let risk_json: serde_json::Value =
        serde_json::from_str(&risk_txt).context("parse /risk/score JSON")?;
    let score_val = risk_json.get("score").and_then(|v| v.as_u64()).unwrap_or(0);
    tracing::info!(%score_val, elapsed_ms = %t_step.elapsed().as_millis(), "passo 1: User Score calculado (antifraude)");
    step_times.push(("User Score (POST /risk/score)", t_step.elapsed()));

    // 2) Pedido protegido: inicia on-ramp e fica PendingFunding
    let t_step = Instant::now();
    let order_body = json!({
        "buyer_id": buyer,
        "seller_id": seller,
        "amount": amount,
        "cpf": cpf,
        "social_links": ["https://instagram.com/old_user"],
        "description": "CLI test-flow — pedido protegido",
    });

    let res = authed_post(&client, "/orders", &order_body, buyer_token)
        .await
        .context("POST /orders")?;
    let status = res.status();
    let body = res.text().await?;
    if !status.is_success() {
        anyhow::bail!("POST /orders failed: {status} — {body}");
    }
    let v: serde_json::Value = serde_json::from_str(&body).context("parse order response")?;
    let order_id: Uuid = serde_json::from_value(
        v.get("id")
            .cloned()
            .context("missing id in order response")?,
    )
    .context("order id uuid")?;
    let funding_tx = v
        .get("gateway_in_tx_id")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    let soroban_mode = v
        .get("soroban_mode")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    tracing::info!(
        %order_id,
        %funding_tx,
        %soroban_mode,
        risk = %v.get("risk_score").unwrap_or(&json!(null)),
        elapsed_ms = %t_step.elapsed().as_millis(),
        "passo 2: pedido — funding iniciado (pending_funding)"
    );
    step_times.push(("Pedido pendente funding (POST /orders)", t_step.elapsed()));

    // 3) Settlement + lock
    let t_step = Instant::now();
    let settle_body = json!({
        "order_id": order_id,
    });
    let settle_url = format!("{}/internal/orders/settle", base.trim_end_matches('/'));
    let mut settle_req = client.post(settle_url).json(&settle_body);
    if let Ok(k) = std::env::var("APICASH_API_KEY") {
        if !k.trim().is_empty() {
            settle_req = settle_req.header("x-api-key", k);
        }
    } else if let Some(t) = buyer_token {
        settle_req = settle_req.header("Authorization", format!("Bearer {t}"));
    }
    let settle_res = settle_req
        .send()
        .await
        .context("POST /internal/orders/settle")?;
    let settle_status = settle_res.status();
    let settle_txt = settle_res.text().await?;
    if !settle_status.is_success() {
        anyhow::bail!("POST /internal/orders/settle failed: {settle_status} — {settle_txt}");
    }
    let settle_json: serde_json::Value =
        serde_json::from_str(&settle_txt).context("parse settle response")?;
    tracing::info!(
        settle = %settle_json,
        elapsed_ms = %t_step.elapsed().as_millis(),
        "passo 3: funding confirmado e custódia travada"
    );
    step_times.push((
        "Settle funding (POST /internal/orders/settle)",
        t_step.elapsed(),
    ));

    // 4) Tentativa de liberação como vendedor (deve falhar: apenas buyer pode liberar)
    if let Some(t_seller) = seller_token {
        let t_step = Instant::now();
        let bad_release = json!({
            "order_id": order_id,
            "released_by": seller,
            "idempotency_key": format!("cli-seller-{}", Uuid::new_v4()),
        });
        let r = authed_post(&client, "/custody/release", &bad_release, Some(t_seller))
            .await
            .context("POST /custody/release (seller)")?;
        let st = r.status();
        let txt = r.text().await.unwrap_or_default();
        tracing::info!(
            %st,
            elapsed_ms = %t_step.elapsed().as_millis(),
            "passo 4: tentativa de release como vendedor (esperado: 401/403)"
        );
        step_times.push((
            "Seller release denied (POST /custody/release)",
            t_step.elapsed(),
        ));
        if st.is_success() {
            anyhow::bail!("seller release unexpectedly succeeded: {txt}");
        }
    }

    // 5) Confirmação de entrega → libera custódia, calcula e distribui yield
    let t_step = Instant::now();
    let release_body = json!({
        "order_id": order_id,
        "released_by": buyer,
        "idempotency_key": format!("cli-test-{}", Uuid::new_v4()),
    });

    let rel_res = authed_post(&client, "/custody/release", &release_body, buyer_token)
        .await
        .context("POST /custody/release")?;
    let rel_status = rel_res.status();
    let rel_txt = rel_res.text().await?;
    if !rel_status.is_success() {
        anyhow::bail!("POST /custody/release failed: {rel_status} — {rel_txt}");
    }
    let rel_json: serde_json::Value =
        serde_json::from_str(&rel_txt).context("parse release JSON")?;
    tracing::info!(
        yield_distributed = %rel_json.get("yield_distributed").unwrap_or(&json!(null)),
        elapsed_ms = %t_step.elapsed().as_millis(),
        "passo 5: confirmação como comprador — custódia liberada, yield 70/10/20 distribuído"
    );
    step_times.push((
        "Buyer confirm + yield (POST /custody/release)",
        t_step.elapsed(),
    ));

    // 6) Off-ramp (BRLx → PIX) — mock por padrão
    let t_step = Instant::now();
    let off_body = json!({
        "destination_pix_key": "cli+offramp@apicash.dev",
    });
    let off_url = format!(
        "{}/orders/{}/off-ramp",
        base.trim_end_matches('/'),
        order_id
    );
    let mut off_req = client.post(off_url).json(&off_body);
    if let Some(t) = buyer_token {
        off_req = off_req.header("Authorization", format!("Bearer {t}"));
    }
    let off_res = off_req.send().await.context("POST /orders/{id}/off-ramp")?;
    let off_status = off_res.status();
    let off_txt = off_res.text().await?;
    if !off_status.is_success() {
        anyhow::bail!("off-ramp failed: {off_status} — {off_txt}");
    }
    let off_json: serde_json::Value =
        serde_json::from_str(&off_txt).context("parse off-ramp JSON")?;
    tracing::info!(
        off = %off_json,
        elapsed_ms = %t_step.elapsed().as_millis(),
        "passo 6: off-ramp mock (saída para PIX)"
    );
    step_times.push((
        "Off-ramp mock (POST /orders/{id}/off-ramp)",
        t_step.elapsed(),
    ));

    let total = t0.elapsed();

    println!();
    println!("══════════════════════════════════════════════════════════════");
    println!(" APICash — resultado do test-flow");
    println!("══════════════════════════════════════════════════════════════");
    println!(" order_id          : {order_id}");
    println!(" comprador / vendedor : {buyer} / {seller}");
    println!(" valor (principal) : {amount} BRL");
    println!(" score (pré-pedido): {score_val}");
    println!(" funding tx        : {funding_tx}");
    println!(" soroban_mode      : {soroban_mode}");
    println!(
        " yield distribuído : {}",
        rel_json.get("yield_distributed").unwrap_or(&json!({}))
    );
    println!(
        " off-ramp          : {}",
        off_json.get("tx_hash").unwrap_or(&json!(""))
    );
    println!("──────────────────────────────────────────────────────────────");
    println!(" Tempos por etapa:");
    for (label, d) in &step_times {
        println!("   • {label}: {} ms", d.as_millis());
    }
    println!(" Total wall-clock  : {} ms", total.as_millis());
    println!("══════════════════════════════════════════════════════════════");
    println!();

    Ok(())
}
