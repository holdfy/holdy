use chrono::Datelike;
// Sulcred Simulator - mock do gateway PIX (porta 7020)
// Faz exatamente o que gateboxgo/simulators/sulcred faz
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::atomic::{AtomicU32, Ordering},
    sync::Arc,
};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::info;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
struct AppState {
    webhook_urls: Arc<RwLock<HashMap<String, String>>>,
    webhook_base_url: String,
    failure_counter: Arc<AtomicU32>,
}

fn hmac_sign(payload: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn check_failure_config_sulcred(headers: &HeaderMap, failure_counter: &AtomicU32) -> Option<(u16, String)> {
    let h = headers.get("X-Gateway-Failure-Config")?.to_str().ok()?;
    let cfg: serde_json::Value = serde_json::from_str(h).ok()?;
    // Formato: {"sulcred": {...}} ou flat {...}
    let sulcred = cfg
        .get("sulcred")
        .cloned()
        .or_else(|| {
            if cfg.get("fail_after_requests").is_some() {
                Some(cfg.clone())
            } else {
                None
            }
        })?;
    let fail_after = sulcred.get("fail_after_requests").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let recover_after = sulcred
        .get("gateway_recover_after_transactions")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let current_fallback = sulcred
        .get("current_fallback_transactions")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let error_code = sulcred
        .get("error_code")
        .and_then(|v| v.as_u64())
        .unwrap_or(500) as u16;
    let error_msg = sulcred
        .get("error_message")
        .and_then(|v| v.as_str())
        .unwrap_or("Gateway temporarily unavailable")
        .to_string();

    if recover_after > 0 && current_fallback >= recover_after {
        return None;
    }
    let count = failure_counter.fetch_add(1, Ordering::SeqCst) + 1;
    if fail_after > 0 && count >= fail_after {
        return Some((error_code, error_msg));
    }
    None
}

fn extract_internal_tx_id(body: &serde_json::Value) -> Option<String> {
    body.get("internalTransactionId")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            body.get("internalTransactionId")
                .and_then(|v| v.as_f64())
                .map(|n| n as i64)
                .map(|n| n.to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_internal_tx_id() {
        let body = serde_json::json!({"internalTransactionId": "abc123"});
        assert_eq!(extract_internal_tx_id(&body), Some("abc123".to_string()));
        let body_num = serde_json::json!({"internalTransactionId": 999});
        assert_eq!(extract_internal_tx_id(&body_num), Some("999".to_string()));
        let body_missing = serde_json::json!({"other": "x"});
        assert_eq!(extract_internal_tx_id(&body_missing), None);
    }
}

fn extract_amount(body: &serde_json::Value) -> f64 {
    body.get("payment")
        .and_then(|p| p.get("amount"))
        .and_then(|a| a.as_f64())
        .unwrap_or_else(|| {
            body.get("payment")
                .and_then(|p| p.get("amount"))
                .and_then(|a| a.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0)
        })
}

fn extract_pix_key(body: &serde_json::Value) -> String {
    body.get("pixKey")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn generate_sulcred_pix_response(body: &serde_json::Value) -> serde_json::Value {
    let now = chrono::Utc::now();
    let unix = now.timestamp();
    let id = unix % 1_000_000_000;
    let amount = extract_amount(body);
    let pix_key = extract_pix_key(body);
    let end_to_end = format!(
        "E{:04}{:02}{:02}{:010}",
        now.year(),
        now.month(),
        now.day(),
        unix % 10_000_000_000
    );

    serde_json::json!({
        "endToEndId": end_to_end,
        "eventDate": now.to_rfc3339(),
        "status": "processing",
        "id": id,
        "payment": {
            "currency": "BRL",
            "amount": format!("{:.2}", amount)
        },
        "type": "PIX_OUT",
        "data": {
            "id": id,
            "refunds": [],
            "idempotencyKey": format!("idemp_{}", unix),
            "endToEndId": end_to_end,
            "pixKey": pix_key,
            "payment": {
                "currency": "BRL",
                "amount": amount
            },
            "status": "processing",
            "transactionType": "PIX",
            "localInstrument": "DICT",
            "createdAt": now.to_rfc3339(),
            "creditorAccount": {
                "ispb": "00000000",
                "issuer": "Banco Teste",
                "number": "12345",
                "accountType": "CACC",
                "document": "12345678901",
                "name": "Test Creditor"
            },
            "debtorAccount": {
                "ispb": "00000000",
                "issuer": "Banco Teste",
                "number": "67890",
                "accountType": "CACC",
                "document": "98765432100",
                "name": "Test Debtor"
            },
            "remittanceInformation": "Test PIX",
            "errorCode": "",
            "txId": format!("tx_{}", unix),
            "creditDebitType": "DEBIT"
        }
    })
}

async fn simulate_pix_op(
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
    operation: &str,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let operation = operation.to_string();
    if let Some((code, msg)) = check_failure_config_sulcred(&headers, &s.failure_counter) {
        return Err((
            axum::http::StatusCode::from_u16(code)
                .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
            Json(serde_json::json!({
                "error": {"code": code, "message": msg},
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        ));
    }

    let response = generate_sulcred_pix_response(&body);

    if operation == "send_key" || operation == "send_qrcode" {
        let internal_id = extract_internal_tx_id(&body);
        let base_url = s.webhook_base_url.clone();
        let urls = s.webhook_urls.clone();
        let resp_clone = response.clone();
        tokio::spawn(async move {
            send_pix_out_webhook(internal_id, base_url, urls, resp_clone, &operation).await;
        });
    }

    Ok(Json(response))
}

async fn send_pix_out_webhook(
    internal_id: Option<String>,
    base_url: String,
    urls: Arc<RwLock<HashMap<String, String>>>,
    response: serde_json::Value,
    _operation: &str,
) {
    let internal_id = match internal_id {
        Some(id) if !id.is_empty() => id,
        _ => {
            tracing::debug!("Sulcred - Sem internalTransactionId: webhook não enviado");
            return;
        }
    };

    let webhook_url = urls
        .read()
        .await
        .get("transfer")
        .cloned()
        .unwrap_or_else(|| format!("{}/api/v1/pix/webhook/out", base_url));

    let simulate_400_rate: f64 = env::var("SULCRED_SIMULATE_WEBHOOK_400_RATE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let simulate_fail_rate: f64 = env::var("SULCRED_SIMULATE_WEBHOOK_FAILURE_RATE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    let (status, message, include_internal) = if simulate_400_rate > 0.0 && rand::random::<f64>() < simulate_400_rate {
        ("failed".to_string(), "Simulated 400".to_string(), false)
    } else if simulate_fail_rate > 0.0 && rand::random::<f64>() < simulate_fail_rate {
        (
            "failed".to_string(),
            "Simulated failure for testing (SULCRED_SIMULATE_WEBHOOK_FAILURE_RATE)".to_string(),
            true,
        )
    } else {
        (
            "completed".to_string(),
            "PIX processed successfully".to_string(),
            true,
        )
    };

    let tx_id = response
        .get("id")
        .and_then(|v| v.as_i64())
        .map(|n| n.to_string())
        .unwrap_or_default();
    let end_to_end = response
        .get("endToEndId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let amount = response
        .get("data")
        .and_then(|d| d.get("payment"))
        .and_then(|p| p.get("amount"))
        .and_then(|a| a.as_f64())
        .unwrap_or(0.0);

    let mut payload = serde_json::json!({
        "transactionId": tx_id,
        "endToEndId": end_to_end,
        "status": status,
        "amount": amount,
        "gatewayName": "sulcred",
        "message": message,
        "processedAt": chrono::Utc::now().to_rfc3339(),
        "internalTransactionId": ""
    });
    if include_internal {
        payload["internalTransactionId"] = serde_json::Value::String(internal_id);
    }

    let json_str = payload.to_string();
    let signature = hmac_sign(&json_str, "webhook_secret_key_simulator");

    let client = reqwest::Client::new();
    let resp = client
        .post(&webhook_url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-Signature", signature)
        .header("X-Idempotency-Key", &end_to_end)
        .body(json_str)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            tracing::info!(
                "Sulcred - Webhook enviado: transaction_id={}, status={}",
                tx_id,
                status
            );
        }
        Ok(r) => {
            tracing::warn!(
                "Sulcred - Webhook retornou {}: {:?}",
                r.status(),
                r.text().await
            );
        }
        Err(e) => {
            tracing::warn!("Sulcred - Erro ao enviar webhook: {}", e);
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let port = env::var("PORT").unwrap_or_else(|_| "7020".to_string());
    let mut webhook_base_url =
        env::var("GATEBOXGO_WEBHOOK_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    if webhook_base_url.contains("localhost") || webhook_base_url.contains("127.0.0.1") {
        webhook_base_url = "http://localhost:8080".to_string();
    }
    webhook_base_url = webhook_base_url.trim_end_matches('/').to_string();

    let state = AppState {
        webhook_urls: Arc::new(RwLock::new(HashMap::new())),
        webhook_base_url: webhook_base_url.clone(),
        failure_counter: Arc::new(AtomicU32::new(0)),
    };
    {
        let mut urls = state.webhook_urls.write().await;
        urls.insert(
            "transfer".to_string(),
            format!("{}/api/v1/pix/webhook/out", webhook_base_url),
        );
    }

    info!("Sulcred Simulator - Webhook base: {}", webhook_base_url);

    let app = Router::new()
        .route("/health", get(health))
        .route("/oauth/token", post(auth_pix_in))
        .route("/api/v2/oauth/token", post(auth_pix_out))
        // Alias `/cob` — gatebox-rust `SulcredHttpService::create_dynamic_qrcode` faz POST `{SULCRED_OUT_URL}/cob`
        .route("/cob", post(create_dynamic_qrcode))
        .route("/api/v2/pix/keys", post(send_pix_key))
        .route("/api/v2/pix/payments/dict", post(send_pix_key))
        .route("/api/v2/pix/qrcode", post(send_pix_qrcode))
        .route("/api/v2/pix/payments/qrc", post(send_pix_qrcode))
        .route("/api/v2/pix/qrcode/dynamic", post(create_dynamic_qrcode))
        .route("/api/v2/pix/pac004", post(pac004))
        .route("/api/v2/account/balance", get(get_balance))
        .route("/api/v2/account/extract", get(get_extract))
        .route("/api/v2/transactions/:id", get(get_transaction_details))
        .route("/api/v2/transactions/pix/:endtoend", get(get_pix_by_endtoend))
        .route(
            "/api/v2/transactions/pix/idempotency/:key",
            get(get_pix_by_idempotency),
        )
        .route("/api/v2/webhooks/pix/in", post(webhook_in))
        .route("/api/v2/webhooks/pix/out/transfer", post(webhook_transfer))
        .route("/api/v2/webhooks/pix/out/receive", post(webhook_receive))
        .route("/api/v2/webhooks/pix/out/refund", post(webhook_refund))
        .route("/api/v2/webhooks/pix/out/cashout", post(webhook_cashout))
        .route("/api/v2/webhooks/pix/out/infraction", post(webhook_infraction))
        .route("/api/v2/med/", get(list_meds))
        .route("/api/v2/med/:id", get(get_med).post(respond_med))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    let use_tls = env::var("SULCRED_USE_TLS").unwrap_or_else(|_| "false".to_string());
    let use_tls = use_tls != "false" && use_tls != "0";

    if use_tls {
        let cert_path = env::var("SULCRED_TLS_CERT_PATH").unwrap_or_else(|_| "cert.pem".to_string());
        let key_path = env::var("SULCRED_TLS_KEY_PATH").unwrap_or_else(|_| "key.pem".to_string());
        if let Ok(config) = axum_server::tls_rustls::RustlsConfig::from_pem_file(&cert_path, &key_path).await {
            info!("Sulcred Simulator iniciado em https://{} (TLS)", addr);
            axum_server::bind_rustls(addr, config).serve(app.into_make_service()).await?;
        } else {
            tracing::warn!("SULCRED_USE_TLS=true mas cert/key não encontrados ({} / {}); usando HTTP", cert_path, key_path);
            info!("Sulcred Simulator iniciado em http://{} (fallback HTTP)", addr);
            axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
        }
    } else {
        info!("Sulcred Simulator iniciado em http://{}", addr);
        axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    }
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "Sulcred Simulator",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn auth_pix_in(Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let token = format!("sc_PIX_IN_{}", chrono::Utc::now().timestamp());
    Json(serde_json::json!({
        "access_token": token,
        "token_type": "Bearer",
        "expires_in": 3600
    }))
}

async fn auth_pix_out(Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let token = format!("sc_PIX_OUT_{}", chrono::Utc::now().timestamp());
    Json(serde_json::json!({
        "access_token": token,
        "token_type": "Bearer",
        "expires_in": 3600
    }))
}

async fn send_pix_key(
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    simulate_pix_op(State(s), headers, Json(body), "send_key").await
}

async fn send_pix_qrcode(
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    simulate_pix_op(State(s), headers, Json(body), "send_qrcode").await
}

async fn create_dynamic_qrcode(Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let txid = format!("{:0>32}", chrono::Utc::now().timestamp_millis());
    let pix_copia_cola = format!(
        "00020126580014br.gov.bcb.pix0136{}520400005303986540510.005802BR5913Simulator6009SAO PAULO62070503***6304",
        uuid::Uuid::new_v4()
    );
    Json(serde_json::json!({
        "txid": txid,
        "location": format!("/v2/cob/{}", txid),
        "pixCopiaECola": pix_copia_cola
    }))
}

async fn pac004(Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn get_balance() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "balance": {
            "available": 15000.50,
            "blocked": 500.00,
            "total": 15500.50
        },
        "currency": "BRL",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_extract() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "transactions": [],
        "total_count": 0,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_transaction_details(Path(_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": _id,
        "type": "pix_transaction",
        "status": "completed",
        "amount": 100.00,
        "currency": "BRL",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_pix_by_endtoend(Path(_endtoend): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "end_to_end_id": _endtoend,
        "status": "completed",
        "amount": 100.00,
        "currency": "BRL",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_pix_by_idempotency(Path(_key): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "idempotency_key": _key,
        "status": "completed",
        "amount": 100.00,
        "currency": "BRL",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "transaction_id": format!("sulcred_tx_{}", _key)
    }))
}

async fn webhook_in(State(s): State<AppState>, Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    if let Some(url) = body.get("url").and_then(|v| v.as_str()) {
        s.webhook_urls.write().await.insert("pix_in".to_string(), url.to_string());
    }
    Json(serde_json::json!({"status": "registered"}))
}

async fn webhook_transfer(State(s): State<AppState>, Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    if let Some(url) = body.get("url").and_then(|v| v.as_str()) {
        s.webhook_urls.write().await.insert("transfer".to_string(), url.to_string());
    }
    Json(serde_json::json!({"status": "registered"}))
}

async fn webhook_receive(State(_s): State<AppState>, Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "registered"}))
}

async fn webhook_refund(State(_s): State<AppState>, Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "registered"}))
}

async fn webhook_cashout(State(_s): State<AppState>, Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "registered"}))
}

async fn webhook_infraction(State(_s): State<AppState>, Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "registered"}))
}

async fn list_meds() -> Json<serde_json::Value> {
    Json(serde_json::json!({"meds": []}))
}

async fn get_med(Path(_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"id": _id, "status": "pending"}))
}

async fn respond_med(Path(_id): Path<String>, Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "responded"}))
}
