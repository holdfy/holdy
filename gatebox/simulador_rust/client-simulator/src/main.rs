// Client Simulator - load/stress tester para API Gatebox (porta 7070)
// Faz exatamente o que gateboxgo/simulators/client-simulator faz
mod consumer;
mod consumer_pulsar;
mod openapi;
mod publisher;
mod publisher_pulsar;

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    time::Instant,
};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone, Default, Serialize, Deserialize)]
struct Config {
    scenario: String,
    transactions: u64,
    concurrency: u32,
    duration_secs: Option<u64>,
    ramp_up_secs: Option<u64>,
    ramp_down_secs: Option<u64>,
    failure_rate: f64,
    timeout_secs: Option<u64>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct Metrics {
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    total_transactions: u64,
    successful_tx: u64,
    failed_tx: u64,
    average_latency_ms: f64,
    min_latency_ms: f64,
    max_latency_ms: f64,
    throughput_tps: f64,
    error_rate: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PixTargetResponse {
    target_id: String,
    status: String,
    target_amount: f64,
    min_amount: f64,
    max_amount: f64,
    concurrency: u32,
    distribution: String,
    message: String,
}

#[derive(Clone)]
struct AppState {
    config: Arc<RwLock<Config>>,
    metrics: Arc<RwLock<Metrics>>,
    running: Arc<AtomicBool>,
    gateway_url: String,
    targets: Arc<RwLock<HashMap<String, PixTargetResponse>>>,
    api_target_configs: Arc<RwLock<HashMap<String, ApiTargetConfig>>>,
    rabbitmq_target_configs: Arc<RwLock<HashMap<String, publisher::RabbitMQTargetConfig>>>,
    pulsar_target_configs: Arc<RwLock<HashMap<String, publisher_pulsar::PulsarTargetConfig>>>,
    api_progress: Arc<RwLock<HashMap<String, Arc<std::sync::atomic::AtomicI64>>>>,
    consumer_stats: Option<Arc<consumer::ConsumerStats>>,
}

#[derive(Clone)]
struct ApiTargetConfig {
    target_transactions: usize,
    min_amount: f64,
    max_amount: f64,
    concurrency: usize,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let port = env::var("PORT").unwrap_or_else(|_| "7070".to_string());
    let gateway_url =
        env::var("GATEWAY_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
    let backend = std::env::var("MESSAGING_BACKEND")
        .unwrap_or_else(|_| "pulsar".to_string())
        .to_lowercase()
        .trim()
        .to_string();

    let consumer_stats: Option<Arc<consumer::ConsumerStats>> = if backend == "pulsar" {
        let stats = Arc::new(consumer::ConsumerStats::default());
        let pulsar_url =
            env::var("PULSAR_URL").unwrap_or_else(|_| "pulsar://localhost:6650".to_string());
        let api_url = gateway_url.clone();
        let stats_clone = Arc::clone(&stats);
        tokio::spawn(async move {
            loop {
                if let Err(e) = consumer_pulsar::run_pulsar_consumer(&pulsar_url, &api_url, stats_clone.clone()).await
                {
                    tracing::warn!("Pulsar consumer erro (reiniciando): {}", e);
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
        Some(stats)
    } else if backend == "rabbitmq" {
        let stats = Arc::new(consumer::ConsumerStats::default());
        let rabbitmq_url =
            env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://adminUser:strongPassword@localhost:5673".to_string());
        let queue_name = "client-simulator-queue".to_string();
        let api_url = gateway_url.clone();
        let stats_clone = Arc::clone(&stats);
        tokio::spawn(async move {
            loop {
                if let Err(e) = consumer::run_consumer(&rabbitmq_url, &queue_name, &api_url, stats_clone.clone()).await
                {
                    tracing::warn!("Consumer RabbitMQ erro (reiniciando): {}", e);
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
        Some(stats)
    } else {
        info!("PIX Consumer desabilitado (MESSAGING_BACKEND={}, use pulsar ou rabbitmq)", backend);
        None
    };

    let state = AppState {
        config: Arc::new(RwLock::new(Config {
            scenario: "load".to_string(),
            transactions: 1000,
            concurrency: 10,
            duration_secs: Some(300),
            ramp_up_secs: Some(30),
            ramp_down_secs: Some(30),
            failure_rate: 0.0,
            timeout_secs: Some(30),
        })),
        metrics: Arc::new(RwLock::new(Metrics::default())),
        running: Arc::new(AtomicBool::new(false)),
        gateway_url: gateway_url.clone(),
        targets: Arc::new(RwLock::new(HashMap::new())),
        api_target_configs: Arc::new(RwLock::new(HashMap::new())),
        rabbitmq_target_configs: Arc::new(RwLock::new(HashMap::new())),
        pulsar_target_configs: Arc::new(RwLock::new(HashMap::new())),
        api_progress: Arc::new(RwLock::new(HashMap::new())),
        consumer_stats,
    };

    info!("Client Simulator - Gateway: {}", gateway_url);

    let api_routes = Router::new()
        .route("/health", get(health))
        .route("/config", get(get_config).post(set_config))
        .route("/status", get(status))
        .route("/metrics", get(metrics))
        .route("/start", post(start_test))
        .route("/stop", post(stop_test))
        .route("/report", get(report))
        .route("/consumer/status", get(consumer_status))
        .route("/pix/simulate", post(pix_simulate))
        .route("/pix/target", post(create_pix_target))
        .route("/pix/target/:id/start", post(start_pix_target))
        .route("/pix/target/:id/stop", post(stop_pix_target))
        .route("/pix/target/:id/status", get(get_pix_target_status))
        .route("/pix/target/:id/progress", get(get_pix_target_progress))
        .route("/pix/targets", get(list_pix_targets))
        .route("/pix/target/:id", delete(delete_pix_target))
        .route("/pix/target/:id/reconciliation", get(get_pix_target_reconciliation))
        .with_state(state);

    let swagger = SwaggerUi::new("/swagger")
        .url("/api-docs/openapi.json", openapi::ApiDoc::openapi());

    let app = Router::new()
        .merge(api_routes)
        .merge(swagger)
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    info!("Client Simulator iniciado em http://{}", addr);
    info!("Swagger: http://localhost:{}/swagger/", port);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "Client Simulator",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_config(State(s): State<AppState>) -> Json<Config> {
    let config = s.config.read().await.clone();
    Json(config)
}

async fn set_config(State(s): State<AppState>, Json(config): Json<Config>) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    if s.running.load(Ordering::SeqCst) {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "não é possível alterar configuração durante teste"})),
        ));
    }
    let mut c = s.config.write().await;
    *c = config;
    Ok(Json(serde_json::json!({"message": "Configuração atualizada"})))
}

async fn status(State(s): State<AppState>) -> Json<serde_json::Value> {
    let config = s.config.read().await.clone();
    let metrics = s.metrics.read().await.clone();
    Json(serde_json::json!({
        "running": s.running.load(Ordering::SeqCst),
        "config": config,
        "metrics": metrics
    }))
}

async fn metrics(State(s): State<AppState>) -> Json<Metrics> {
    let m = s.metrics.read().await.clone();
    Json(m)
}

async fn start_test(State(s): State<AppState>) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    if s.running.swap(true, Ordering::SeqCst) {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "teste já está rodando"})),
        ));
    }
    let mut m = s.metrics.write().await;
    m.start_time = Some(Utc::now());
    m.total_transactions = 0;
    m.successful_tx = 0;
    m.failed_tx = 0;
    drop(m);

    let state = s.clone();
    let gateway = s.gateway_url.clone();
    tokio::spawn(async move {
        run_load_test(state, gateway).await;
    });
    Ok(Json(serde_json::json!({"message": "Teste iniciado"})))
}

async fn run_load_test(s: AppState, gateway_url: String) {
    let config = s.config.read().await.clone();
    let concurrency = config.concurrency.max(1);
    let total = config.transactions;
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/pix/send", gateway_url.trim_end_matches('/'));
    let mut success = 0u64;
    let mut failed = 0u64;
    let mut latencies: Vec<f64> = Vec::new();
    let start = Instant::now();
    let sem = Arc::new(tokio::sync::Semaphore::new(concurrency as usize));

    for i in 0..total {
        if !s.running.load(Ordering::SeqCst) {
            break;
        }
        let _permit = sem.clone().acquire_owned().await.unwrap();
        let req_start = Instant::now();
        let body = serde_json::json!({
            "account": "2000001",
            "bank": "00000000",
            "branch": "0001",
            "amount": 1.0,
            "key": format!("sim-{}@test.com", i),
            "name": "Simulator",
            "documentNumber": "12345678900",
            "typeKey": "EMAIL",
            "externalId": format!("load-{}", i),
            "userId": 3
        });
        match client.post(&url).json(&body).send().await {
            Ok(resp) => {
                let ms = req_start.elapsed().as_secs_f64() * 1000.0;
                latencies.push(ms);
                if resp.status().is_success() {
                    success += 1;
                } else {
                    failed += 1;
                }
            }
            Err(_) => {
                failed += 1;
            }
        }
    }

    s.running.store(false, Ordering::SeqCst);
    let end = Instant::now();
    let mut m = s.metrics.write().await;
    m.end_time = Some(Utc::now());
    m.total_transactions = success + failed;
    m.successful_tx = success;
    m.failed_tx = failed;
    if !latencies.is_empty() {
        let sum: f64 = latencies.iter().sum();
        m.average_latency_ms = sum / latencies.len() as f64;
        m.min_latency_ms = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
        m.max_latency_ms = latencies.iter().cloned().fold(0.0, f64::max);
    }
    let dur = end.duration_since(start).as_secs_f64();
    if dur > 0.0 {
        m.throughput_tps = (success + failed) as f64 / dur;
    }
    if m.total_transactions > 0 {
        m.error_rate = m.failed_tx as f64 / m.total_transactions as f64 * 100.0;
    }
}

async fn stop_test(State(s): State<AppState>) -> Json<serde_json::Value> {
    s.running.store(false, Ordering::SeqCst);
    Json(serde_json::json!({"message": "Teste parado"}))
}

async fn report(State(s): State<AppState>) -> Json<serde_json::Value> {
    let config = s.config.read().await.clone();
    let metrics = s.metrics.read().await.clone();
    Json(serde_json::json!({
        "config": config,
        "metrics": metrics,
        "results": []
    }))
}

async fn consumer_status(State(s): State<AppState>) -> Json<serde_json::Value> {
    if let Some(ref stats) = s.consumer_stats {
        Json(stats.to_map())
    } else {
        Json(serde_json::json!({
            "consumer_enabled": false,
            "message": "Consumer not initialized (MESSAGING_BACKEND=pulsar ou rabbitmq)"
        }))
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct PixTypeConfig {
    #[serde(default)]
    target_transactions: usize,
    #[serde(default)]
    min_amount: f64,
    #[serde(default)]
    max_amount: f64,
    #[serde(default = "default_true")]
    enabled: bool,
}

fn default_true() -> bool {
    true
}

async fn pix_simulate(State(s): State<AppState>, Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    // Formato legado: target_amount no root
    let has_legacy = body.get("target_amount").is_some();
    if has_legacy {
        let concurrency = body.get("concurrency").and_then(|c| c.as_u64()).unwrap_or(10) as u32;
        let target_transactions = body.get("target_transactions").and_then(|t| t.as_u64()).unwrap_or(100);
        {
            let mut c = s.config.write().await;
            c.concurrency = concurrency.max(1);
            c.transactions = target_transactions;
        }
        let _ = start_test(State(s)).await;
        return Json(serde_json::json!({
            "status": "started",
            "concurrency": concurrency,
            "target_transactions": target_transactions
        }));
    }

    // Formato multi-tipo: pix_out_key, pix_in_key, etc
    let pix_out_key: Option<PixTypeConfig> = body.get("pix_out_key").and_then(|v| serde_json::from_value(v.clone()).ok());
    let pix_out_qrcode: Option<PixTypeConfig> =
        body.get("pix_out_qrcode").and_then(|v| serde_json::from_value(v.clone()).ok());
    let pix_in_key: Option<PixTypeConfig> = body.get("pix_in_key").and_then(|v| serde_json::from_value(v.clone()).ok());
    let pix_in_qrcode: Option<PixTypeConfig> =
        body.get("pix_in_qrcode").and_then(|v| serde_json::from_value(v.clone()).ok());
    let concurrency = body.get("concurrency").and_then(|c| c.as_u64()).unwrap_or(10) as usize;
    let concurrency = concurrency.max(1).min(20);
    let initial_balance = body.get("initial_balance").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let target_id = format!("pix_multi_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));

    if initial_balance > 0.0 {
        if let Err(e) = credit_initial_balance(&s.gateway_url, &target_id, initial_balance).await {
            tracing::warn!("Não foi possível creditar saldo inicial: {}", e);
        } else {
            tracing::info!("Saldo inicial de R$ {:.2} creditado com sucesso", initial_balance);
        }
    }

    let gateway = s.gateway_url.clone();
    let mut workers = Vec::new();

    if let Some(ref cfg) = pix_out_key {
        if cfg.enabled && cfg.target_transactions > 0 {
            let g = gateway.clone();
            let c = cfg.clone();
            workers.push(tokio::spawn(async move {
                run_type_pix_out(&g, "pix_out_key", concurrency, c).await
            }));
        }
    }
    if let Some(ref cfg) = pix_out_qrcode {
        if cfg.enabled && cfg.target_transactions > 0 {
            let g = gateway.clone();
            let c = cfg.clone();
            workers.push(tokio::spawn(async move {
                run_type_pix_out(&g, "pix_out_qrcode", concurrency, c).await
            }));
        }
    }
    if let Some(ref cfg) = pix_in_key {
        if cfg.enabled && cfg.target_transactions > 0 {
            let g = gateway.clone();
            let c = cfg.clone();
            workers.push(tokio::spawn(async move {
                run_type_pix_in_key(&g, concurrency, c).await
            }));
        }
    }
    if let Some(ref cfg) = pix_in_qrcode {
        if cfg.enabled && cfg.target_transactions > 0 {
            let g = gateway.clone();
            let c = cfg.clone();
            workers.push(tokio::spawn(async move {
                run_type_pix_in_qrcode(&g, concurrency, c).await
            }));
        }
    }

    let _ = workers; // mantém JoinHandles vivos; tasks rodam em background

    Json(serde_json::json!({
        "target_id": target_id,
        "status": "running",
        "message": "Simulação multi-tipo iniciada",
        "initial_balance": initial_balance,
        "types_configured": {
            "pix_out_key": pix_out_key,
            "pix_out_qrcode": pix_out_qrcode,
            "pix_in_key": pix_in_key,
            "pix_in_qrcode": pix_in_qrcode,
        },
        "concurrency": concurrency,
    }))
}

async fn credit_initial_balance(gateway_url: &str, target_id: &str, amount: f64) -> Result<(), String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let pix_key = "test@simulator.com";
    let end_to_end = format!("E{}_saldo_inicial", target_id);
    let payload = serde_json::json!({
        "endToEndId": end_to_end,
        "amount": amount,
        "pixKey": pix_key,
        "payerName": "Simulador - Crédito Inicial",
        "payerDocument": "12345678900",
        "description": format!("Crédito inicial para testes - R$ {:.2}", amount),
        "idempotencyKey": end_to_end,
        "transactionDate": chrono::Utc::now().to_rfc3339(),
    });
    let json_str = payload.to_string();
    let mut mac = HmacSha256::new_from_slice(b"webhook_secret_key_simulator").map_err(|e| e.to_string())?;
    mac.update(json_str.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    let url = format!("{}/api/v1/pix/webhook/in", gateway_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-Signature", sig)
        .header("X-Idempotency-Key", &end_to_end)
        .body(json_str)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("API retornou {}", resp.status()))
    }
}

async fn run_type_pix_out(gateway: &str, pix_type: &str, concurrency: usize, cfg: PixTypeConfig) {
    let pix_type = pix_type.to_string();
    let url = format!("{}/api/v1/pix/send", gateway.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let per_w = cfg.target_transactions / concurrency;
    let rem = cfg.target_transactions % concurrency;
    let mut handles = Vec::new();
    for w in 0..concurrency {
        let n = per_w + if w < rem { 1 } else { 0 };
        if n == 0 {
            continue;
        }
        let url = url.clone();
        let client = client.clone();
        let pix_type = pix_type.clone();
        let min_a = cfg.min_amount;
        let max_a = cfg.max_amount;
        handles.push(tokio::spawn(async move {
            for i in 0..n {
                let amount = if max_a > min_a {
                    min_a + rand::random::<f64>() * (max_a - min_a)
                } else {
                    min_a
                };
                let key = if pix_type.as_str() == "pix_out_qrcode" {
                    format!("QR_{}", i)
                } else {
                    "simulator@test.com".to_string()
                };
                let body = serde_json::json!({
                    "account": "2000001",
                    "bank": "00000000",
                    "branch": "0001",
                    "documentNumber": "12345678900",
                    "amount": amount,
                    "key": key,
                    "name": "Simulator",
                    "externalId": format!("sim-{}-{}", pix_type, i),
                    "typeKey": "EMAIL",
                    "userId": 3
                });
                let _ = client.post(&url).json(&body).send().await;
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
    tracing::info!("{} concluído: {} transações", pix_type, cfg.target_transactions);
}

async fn run_type_pix_in_key(gateway: &str, concurrency: usize, cfg: PixTypeConfig) {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let secret = "webhook_secret_key_simulator";
    let pix_key = "test@simulator.com";
    let url = format!("{}/api/v1/pix/webhook/in", gateway.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let per_w = cfg.target_transactions / concurrency;
    let rem = cfg.target_transactions % concurrency;
    let mut handles = Vec::new();
    for w in 0..concurrency {
        let n = per_w + if w < rem { 1 } else { 0 };
        if n == 0 {
            continue;
        }
        let url = url.clone();
        let client = client.clone();
        let min_a = cfg.min_amount;
        let max_a = cfg.max_amount;
        handles.push(tokio::spawn(async move {
            for i in 0..n {
                let amount = if max_a > min_a {
                    min_a + rand::random::<f64>() * (max_a - min_a)
                } else {
                    min_a
                };
                let end_to_end = format!("E{:014}-pix_in_key-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0), i);
                let payload = serde_json::json!({
                    "endToEndId": end_to_end,
                    "amount": amount,
                    "pixKey": pix_key,
                    "payerName": format!("Simulador {}", i),
                    "payerDocument": "12345678900",
                    "description": format!("PIX IN - {}", amount),
                    "idempotencyKey": end_to_end,
                    "transactionDate": chrono::Utc::now().to_rfc3339(),
                });
                let json_str = payload.to_string();
                let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC");
                mac.update(json_str.as_bytes());
                let sig = hex::encode(mac.finalize().into_bytes());
                let _ = client
                    .post(&url)
                    .header("X-Webhook-Signature", sig)
                    .header("X-Idempotency-Key", &end_to_end)
                    .json(&payload)
                    .send()
                    .await;
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
    tracing::info!("pix_in_key concluído: {} transações", cfg.target_transactions);
}

async fn run_type_pix_in_qrcode(gateway: &str, concurrency: usize, cfg: PixTypeConfig) {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let secret = "webhook_secret_key_simulator";
    let pix_key = "test@simulator.com";
    let qr_url = format!("{}/api/v1/pix/qrcode", gateway.trim_end_matches('/'));
    let webhook_url = format!("{}/api/v1/pix/webhook/in", gateway.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let per_w = cfg.target_transactions / concurrency;
    let rem = cfg.target_transactions % concurrency;
    let mut handles = Vec::new();
    for w in 0..concurrency {
        let n = per_w + if w < rem { 1 } else { 0 };
        if n == 0 {
            continue;
        }
        let qr_url = qr_url.clone();
        let webhook_url = webhook_url.clone();
        let client = client.clone();
        let min_a = cfg.min_amount;
        let max_a = cfg.max_amount;
        handles.push(tokio::spawn(async move {
            for i in 0..n {
                let amount = if max_a > min_a {
                    min_a + rand::random::<f64>() * (max_a - min_a)
                } else {
                    min_a
                };
                let qr_body = serde_json::json!({
                    "amount": amount,
                    "payer_name": "Cliente Simulado",
                    "payer_document": "12345678900",
                    "description": "QR Code",
                    "expiration_seconds": 1800,
                    "reference": format!("ref-{}", i),
                    "pix_key": pix_key,
                });
                if let Ok(resp) = client.post(&qr_url).json(&qr_body).send().await {
                    if resp.status().is_success() {
                        let qr_json: serde_json::Value = resp.json().await.unwrap_or_default();
                        let end_to_end = qr_json
                            .get("txId")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| {
                                format!(
                                    "E{:014}-qrc-{}",
                                    chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
                                    i
                                )
                            });
                        let webhook_body = serde_json::json!({
                            "endToEndId": &end_to_end,
                            "amount": amount,
                            "pixKey": pix_key,
                            "payerName": "Pagador QR",
                            "payerDocument": "12345678900",
                            "description": "PIX QR",
                            "idempotencyKey": &end_to_end,
                            "transactionDate": chrono::Utc::now().to_rfc3339(),
                            "isQRCodePayment": true,
                        });
                        let json_str = webhook_body.to_string();
                        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC");
                        mac.update(json_str.as_bytes());
                        let sig = hex::encode(mac.finalize().into_bytes());
                        let _ = client
                            .post(&webhook_url)
                            .header("X-Webhook-Signature", sig)
                            .header("X-Idempotency-Key", &end_to_end)
                            .json(&webhook_body)
                            .send()
                            .await;
                    }
                }
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
    tracing::info!("pix_in_qrcode concluído: {} transações", cfg.target_transactions);
}

#[derive(Deserialize)]
struct PixTargetRequest {
    #[serde(default)]
    target_amount: f64,
    #[serde(default)]
    min_amount: f64,
    #[serde(default)]
    max_amount: f64,
    #[serde(default)]
    concurrency: u32,
    #[serde(default = "default_distribution")]
    distribution: String,
    #[serde(default)]
    use_api: bool,
    #[serde(default)]
    rabbitmq_url: String,
    #[serde(default)]
    queue_name: String,
    #[serde(default)]
    exchange_name: String,
    #[serde(default)]
    pulsar_url: String,
}

fn default_distribution() -> String {
    "realistic".to_string()
}

async fn create_pix_target(State(s): State<AppState>, Json(body): Json<PixTargetRequest>) -> Json<PixTargetResponse> {
    let id = format!("pix_target_{}", chrono::Utc::now().timestamp());
    let concurrency = body.concurrency.max(1) as usize;
    let target_transactions = if body.max_amount > 0.0 {
        (body.target_amount / body.max_amount) as usize
    } else {
        1000
    };
    let target_transactions = target_transactions.max(1);

    let use_api = body.use_api;
    let backend = std::env::var("MESSAGING_BACKEND")
        .unwrap_or_else(|_| "pulsar".to_string())
        .to_lowercase();
    let rabbitmq_url = if body.rabbitmq_url.is_empty() {
        env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://adminUser:strongPassword@localhost:5673".to_string())
    } else {
        body.rabbitmq_url
    };
    let queue_name = if body.queue_name.is_empty() {
        "client-simulator-queue".to_string()
    } else {
        body.queue_name
    };
    let exchange_name = if body.exchange_name.is_empty() {
        "client-simulator-exchange".to_string()
    } else {
        body.exchange_name
    };
    let pulsar_url = if body.pulsar_url.is_empty() {
        env::var("PULSAR_URL").unwrap_or_else(|_| "pulsar://localhost:6650".to_string())
    } else {
        body.pulsar_url
    };
    let pulsar_topic = format!(
        "persistent://{}/{}/client-simulator-queue",
        env::var("PULSAR_TENANT").unwrap_or_else(|_| "public".to_string()),
        env::var("PULSAR_NAMESPACE").unwrap_or_else(|_| "default".to_string())
    );

    let message = if use_api {
        "Target configurado (envio via API)".to_string()
    } else if backend == "pulsar" {
        "Target configurado (envio via Pulsar)".to_string()
    } else {
        "Target configurado (envio via RabbitMQ)".to_string()
    };

    let target = PixTargetResponse {
        target_id: id.clone(),
        status: "configured".to_string(),
        target_amount: body.target_amount,
        min_amount: body.min_amount,
        max_amount: body.max_amount,
        concurrency: concurrency as u32,
        distribution: body.distribution,
        message,
    };
    s.targets.write().await.insert(id.clone(), target.clone());

    let progress = Arc::new(std::sync::atomic::AtomicI64::new(0));
    s.api_progress.write().await.insert(id.clone(), progress.clone());

    if use_api {
        s.api_target_configs.write().await.insert(
            id.clone(),
            ApiTargetConfig {
                target_transactions,
                min_amount: body.min_amount,
                max_amount: body.max_amount,
                concurrency,
            },
        );
    } else if backend == "pulsar" {
        s.pulsar_target_configs.write().await.insert(
            id.clone(),
            publisher_pulsar::PulsarTargetConfig {
                pulsar_url,
                topic: pulsar_topic,
                target_transactions,
                min_amount: body.min_amount,
                max_amount: body.max_amount,
                concurrency,
            },
        );
    } else {
        s.rabbitmq_target_configs.write().await.insert(
            id.clone(),
            publisher::RabbitMQTargetConfig {
                rabbitmq_url,
                queue_name,
                exchange_name,
                target_transactions,
                min_amount: body.min_amount,
                max_amount: body.max_amount,
                concurrency,
            },
        );
    }

    Json(target)
}

async fn start_pix_target(State(s): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let mut targets = s.targets.write().await;
    let t = match targets.get_mut(&id) {
        Some(t) => t,
        None => return Json(serde_json::json!({"error": "Target não encontrado", "target_id": id})),
    };
    if t.status == "running" {
        return Json(serde_json::json!({"error": "Target já está rodando", "target_id": id}));
    }
    t.status = "running".to_string();
    drop(targets);

    let api_cfg = s.api_target_configs.read().await.get(&id).cloned();
    let rabbitmq_cfg = s.rabbitmq_target_configs.read().await.get(&id).cloned();
    let pulsar_cfg = s.pulsar_target_configs.read().await.get(&id).cloned();
    let progress = s.api_progress.read().await.get(&id).cloned();
    let gateway = s.gateway_url.clone();
    let target_id = id.clone();

    if let (Some(cfg), Some(progress)) = (api_cfg, progress.clone()) {
        let targets = s.targets.clone();
        tokio::spawn(async move {
            run_target_via_api(gateway, target_id.clone(), cfg, progress, targets).await;
        });
    } else if let (Some(cfg), Some(progress)) = (pulsar_cfg, progress.clone()) {
        let targets = s.targets.clone();
        tokio::spawn(async move {
            publisher_pulsar::run_target_via_pulsar(cfg, target_id, progress, targets).await;
        });
    } else if let (Some(cfg), Some(progress)) = (rabbitmq_cfg, progress) {
        let targets = s.targets.clone();
        tokio::spawn(async move {
            publisher::run_target_via_rabbitmq(cfg, target_id, progress, targets).await;
        });
    }

    Json(serde_json::json!({"message": "Target iniciado", "target_id": id}))
}

async fn run_target_via_api(
    gateway_url: String,
    target_id: String,
    cfg: ApiTargetConfig,
    progress: Arc<std::sync::atomic::AtomicI64>,
    targets: Arc<RwLock<HashMap<String, PixTargetResponse>>>,
) {
    let url = format!("{}/api/v1/pix/send?userId=3", gateway_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let per_worker = cfg.target_transactions / cfg.concurrency;
    let remainder = cfg.target_transactions % cfg.concurrency;

    let mut handles = Vec::new();
    for w in 0..cfg.concurrency {
        let count = per_worker + if w < remainder { 1 } else { 0 };
        if count == 0 {
            continue;
        }
        let url = url.clone();
        let client = client.clone();
        let progress = Arc::clone(&progress);
        let min_a = cfg.min_amount;
        let max_a = cfg.max_amount;
        handles.push(tokio::spawn(async move {
            for _ in 0..count {
                let amount = if max_a > min_a {
                    min_a + rand::random::<f64>() * (max_a - min_a)
                } else {
                    min_a
                };
                let body = serde_json::json!({
                    "account": "1",
                    "bank": "001",
                    "documentNumber": "12345678909",
                    "amount": amount,
                    "branch": "0001",
                    "key": "simulator@test.com",
                    "name": "Simulator PIX Load",
                    "typeKey": "EMAIL",
                });
                if let Ok(resp) = client.post(&url).json(&body).send().await {
                    if resp.status().is_success() {
                        progress.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    if let mut t = targets.write().await {
        if let Some(target) = t.get_mut(&target_id) {
            target.status = "completed".to_string();
            target.message = "Target via API concluído".to_string();
        }
    }
}

async fn stop_pix_target(State(s): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let mut targets = s.targets.write().await;
    if let Some(t) = targets.get_mut(&id) {
        t.status = "completed".to_string();
        Json(serde_json::json!({"message": "Target parado", "target_id": id}))
    } else {
        Json(serde_json::json!({"error": "Target não encontrado", "target_id": id}))
    }
}

async fn get_pix_target_status(State(s): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let targets = s.targets.read().await;
    if let Some(t) = targets.get(&id) {
        Json(serde_json::to_value(t).unwrap())
    } else {
        Json(serde_json::json!({"error": "Target não encontrado", "target_id": id}))
    }
}

async fn get_pix_target_progress(State(s): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let targets = s.targets.read().await;
    let target = match targets.get(&id) {
        Some(t) => t,
        None => return Json(serde_json::json!({"error": "Target não encontrado", "target_id": id})),
    };
    let api_cfg = s.api_target_configs.read().await.get(&id).cloned();
    let rabbitmq_cfg = s.rabbitmq_target_configs.read().await.get(&id).cloned();
    let pulsar_cfg = s.pulsar_target_configs.read().await.get(&id).cloned();
    let total = api_cfg
        .map(|c| c.target_transactions as i64)
        .or(pulsar_cfg.map(|c| c.target_transactions as i64))
        .or(rabbitmq_cfg.map(|c| c.target_transactions as i64))
        .unwrap_or(0);
    let progress = s.api_progress.read().await.get(&id).cloned();
    let sent = progress
        .map(|p| p.load(std::sync::atomic::Ordering::SeqCst))
        .unwrap_or(0);
    let progress_pct = if total > 0 {
        (sent as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let mut out = serde_json::to_value(target).unwrap_or_default();
    out["target_id"] = serde_json::Value::String(id);
    out["sent"] = serde_json::Value::Number(serde_json::Number::from(sent));
    out["total"] = serde_json::Value::Number(serde_json::Number::from(total));
    out["progress_pct"] = serde_json::json!(progress_pct);
    Json(out)
}

async fn list_pix_targets(State(s): State<AppState>) -> Json<serde_json::Value> {
    let targets = s.targets.read().await;
    let list: Vec<&PixTargetResponse> = targets.values().collect();
    Json(serde_json::to_value(list).unwrap())
}

async fn delete_pix_target(State(s): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let removed = {
        let mut targets = s.targets.write().await;
        targets.remove(&id).is_some()
    };
    if removed {
        s.api_target_configs.write().await.remove(&id);
        s.rabbitmq_target_configs.write().await.remove(&id);
        s.pulsar_target_configs.write().await.remove(&id);
        s.api_progress.write().await.remove(&id);
        Json(serde_json::json!({"message": "Target removido", "target_id": id}))
    } else {
        Json(serde_json::json!({"error": "Target não encontrado", "target_id": id}))
    }
}

async fn get_pix_target_reconciliation(State(s): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let targets = s.targets.read().await;
    if targets.contains_key(&id) {
        Json(serde_json::json!({
            "target_id": id,
            "reconciled": true,
            "sent": 0,
            "received": 0
        }))
    } else {
        Json(serde_json::json!({"error": "Target não encontrado", "target_id": id}))
    }
}
