//! Serviço principal: transporte **whatsapp-rust** (defeito), webhook Cloud opcional, fila e `MessageHandler`.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;
use whatsapp_cloud_api::models::webhooks::{NotificationMessageType, NotificationPayload};


use apicash_logistics::{CascadingTracker, LogisticsService, MelhorEnvioClient};

use crate::tracking_monitor::TrackingMonitor;

use super::multidevice;
use crate::conversation_store::ConversationStore;
use crate::core_api::CoreApiClient;
use crate::handlers::message_handler::MessageHandler;
use crate::models::WhatsAppEvent;
use crate::outbound::Outbound;

/// Estado compartilhado do webhook Axum (Meta Cloud API — opcional).
#[derive(Clone)]
pub struct WebhookState {
    pub tx: mpsc::Sender<WhatsAppEvent>,
    pub verify_token: String,
}

/// Converte payload Meta em eventos normalizados.
pub fn notification_to_events(payload: &NotificationPayload) -> Vec<WhatsAppEvent> {
    let mut out = Vec::new();
    for entry in &payload.entry {
        for change in &entry.changes {
            if let Some(msgs) = &change.value.messages {
                for m in msgs {
                    let body = match &m.message_type {
                        NotificationMessageType::Text => m.text.as_ref().map(|t| t.body.clone()),
                        NotificationMessageType::Interactive => {
                            m.interactive.as_ref().and_then(|i| {
                                i.button_reply
                                    .as_ref()
                                    .map(|b| b.id.clone())
                                    .or_else(|| i.list_reply.as_ref().map(|l| l.id.clone()))
                            })
                        }
                        NotificationMessageType::Button => {
                            m.button.as_ref().map(|b| b.payload.clone())
                        }
                        _ => None,
                    };

                    if let Some(body) = body {
                        out.push(WhatsAppEvent::new(m.from.clone(), m.id.clone(), body));
                    }
                }
            }
        }
    }
    out
}

async fn webhook_verify(
    State(state): State<WebhookState>,
    Query(q): Query<whatsapp_cloud_api::models::webhooks::VerificationRequest>,
) -> impl IntoResponse {
    if q.mode == "subscribe" && q.verify_token == state.verify_token {
        return (StatusCode::OK, q.challenge).into_response();
    }
    StatusCode::FORBIDDEN.into_response()
}

async fn webhook_post(
    State(state): State<WebhookState>,
    Json(payload): Json<NotificationPayload>,
) -> StatusCode {
    for ev in notification_to_events(&payload) {
        let _ = state.tx.send(ev).await;
    }
    StatusCode::OK
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "apicash-whatsapp"
    }))
}

async fn ready() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ready",
        "service": "apicash-whatsapp"
    }))
}

/// Servidor HTTP do webhook (Meta Cloud API).
pub async fn run_webhook_server(
    bind: &str,
    state: WebhookState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/webhook/whatsapp", get(webhook_verify).post(webhook_post))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "webhook Cloud API listening");
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Clone)]
struct InternalNotifyState {
    handler: Arc<MessageHandler>,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct BankPaymentNotifyRequest {
    order_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct BankPaymentNotifyByRefRequest {
    payment_reference: String,
}

#[derive(Debug, Deserialize)]
struct TrackingStepNotifyRequest {
    seller_phone: String,
    #[serde(default)]
    buyer_phone: Option<String>,
    #[serde(default)]
    order_id: Option<String>,
    tracking_code: String,
    step_label: String,
    description: String,
}

fn internal_api_key_ok(headers: &HeaderMap, expected: &str) -> bool {
    if expected.is_empty() {
        return false;
    }
    let got = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
        });
    got == Some(expected)
}

async fn internal_bank_payment_notify(
    axum::extract::State(state): axum::extract::State<InternalNotifyState>,
    headers: HeaderMap,
    Json(req): Json<BankPaymentNotifyRequest>,
) -> StatusCode {
    if !internal_api_key_ok(&headers, &state.api_key) {
        return StatusCode::UNAUTHORIZED;
    }
    match state.handler.notify_bank_payment(req.order_id).await {
        Ok(()) => StatusCode::OK,
        Err(e) => {
            tracing::warn!(%e, order_id = %req.order_id, "internal bank payment notify failed");
            StatusCode::BAD_REQUEST
        }
    }
}

async fn internal_bank_payment_notify_by_ref(
    axum::extract::State(state): axum::extract::State<InternalNotifyState>,
    headers: HeaderMap,
    Json(req): Json<BankPaymentNotifyByRefRequest>,
) -> StatusCode {
    if !internal_api_key_ok(&headers, &state.api_key) {
        return StatusCode::UNAUTHORIZED;
    }
    let Some(order_id) =
        crate::payment_notify::PaymentNotifyRegistry::parse_order_id_from_reference(
            &req.payment_reference,
        )
    else {
        tracing::warn!(
            ref_len = req.payment_reference.len(),
            "bank payment notify: could not parse order_id from reference"
        );
        return StatusCode::BAD_REQUEST;
    };
    match state.handler.notify_bank_payment(order_id).await {
        Ok(()) => StatusCode::OK,
        Err(e) => {
            tracing::warn!(%e, %order_id, "internal bank payment notify by ref failed");
            StatusCode::BAD_REQUEST
        }
    }
}

async fn internal_tracking_step_notify(
    axum::extract::State(state): axum::extract::State<InternalNotifyState>,
    headers: HeaderMap,
    Json(req): Json<TrackingStepNotifyRequest>,
) -> StatusCode {
    if !internal_api_key_ok(&headers, &state.api_key) {
        return StatusCode::UNAUTHORIZED;
    }
    state
        .handler
        .notify_tracking_step(
            &req.seller_phone,
            req.buyer_phone.as_deref(),
            req.order_id.as_deref(),
            &req.tracking_code,
            &req.step_label,
            &req.description,
        )
        .await;
    StatusCode::OK
}

fn internal_notify_routes(state: InternalNotifyState) -> Router {
    Router::new()
        .route(
            "/internal/bank-payment-notify",
            post(internal_bank_payment_notify),
        )
        .route(
            "/internal/bank-payment-notify-by-ref",
            post(internal_bank_payment_notify_by_ref),
        )
        .route(
            "/internal/tracking-step-notify",
            post(internal_tracking_step_notify),
        )
        .with_state(state)
}

/// `GET /health`, `/ready` e rotas internas (banco → WhatsApp).
pub async fn run_health_only_server(
    bind: &str,
    handler: Arc<MessageHandler>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let api_key = std::env::var("APICASH_API_KEY").unwrap_or_default();
    let internal = InternalNotifyState { handler, api_key };
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .merge(internal_notify_routes(internal));
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "whatsapp agent health + internal notify");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Fachada do agente: fila + [`MessageHandler`]; transporte Rust e/ou webhook.
pub struct WhatsAppService {
    /// Canal para injetar eventos (testes ou webhook Cloud).
    pub incoming: mpsc::Sender<WhatsAppEvent>,
}

impl WhatsAppService {
    /// Carrega [`AgentConfig::from_env`], sobe transporte(s) e o processador de mensagens.
    pub async fn start() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let tx = spawn_agent(AgentConfig::from_env()).await?;
        Ok(Self { incoming: tx })
    }
}

/// `rust` (defeito) — só `whatsapp-rust`. `cloud` — só webhook Meta. `both` — os dois.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaTransport {
    Rust,
    Cloud,
    Both,
}

/// Configuração do agente a partir do ambiente.
#[derive(Clone, Debug)]
pub struct AgentConfig {
    pub core_url: String,
    pub webhook_bind: String,
    pub verify_token: String,
    pub transport: WaTransport,
    /// URI SQLite para `SqliteStore` (ex. `file:apicash_whatsapp.db`).
    pub sqlite_uri: String,
    pub pair_phone: Option<String>,
    pub pair_custom_code: Option<String>,
    pub push_name: Option<String>,
}

impl AgentConfig {
    pub fn from_env() -> Self {
        let transport = match std::env::var("APICASH_WA_TRANSPORT").map(|v| v.to_ascii_lowercase())
        {
            Ok(s) if s == "cloud" => WaTransport::Cloud,
            Ok(s) if s == "both" => WaTransport::Both,
            _ => WaTransport::Rust,
        };

        Self {
            core_url: std::env::var("APICASH_CORE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000".into()),
            webhook_bind: std::env::var("APICASH_WA_WEBHOOK_BIND")
                .unwrap_or_else(|_| "0.0.0.0:8080".into()),
            verify_token: std::env::var("WHATSAPP_VERIFY_TOKEN")
                .unwrap_or_else(|_| "apicash".into()),
            transport,
            sqlite_uri: std::env::var("APICASH_WA_SQLITE_PATH")
                .unwrap_or_else(|_| "file:apicash_whatsapp.db".into()),
            pair_phone: std::env::var("APICASH_WA_PAIR_PHONE")
                .ok()
                .filter(|s| !s.is_empty()),
            pair_custom_code: std::env::var("APICASH_WA_PAIR_CODE")
                .ok()
                .filter(|s| !s.is_empty()),
            push_name: std::env::var("APICASH_WA_PUSH_NAME")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }
}

/// Inicializa handler + fila, transporte **whatsapp-rust** (defeito) e opcionalmente webhook Cloud.
pub async fn spawn_agent(
    cfg: AgentConfig,
) -> Result<mpsc::Sender<WhatsAppEvent>, Box<dyn std::error::Error + Send + Sync>> {
    let (tx, mut rx) = mpsc::channel::<WhatsAppEvent>(256);

    let outbound = Arc::new(match cfg.transport {
        WaTransport::Cloud => Outbound::from_env(),
        WaTransport::Rust | WaTransport::Both => {
            let client = multidevice::start_multidevice_bridge(
                tx.clone(),
                &cfg.sqlite_uri,
                cfg.pair_phone.clone(),
                cfg.pair_custom_code.clone(),
                cfg.push_name.clone(),
            )
            .await?;
            Outbound::Rust {
                client,
                sqlite_uri: Some(cfg.sqlite_uri.clone()),
            }
        }
    });

    let core = CoreApiClient::new(cfg.core_url);
    let sessions = Arc::new(crate::session::SessionManager::new());
    let payment_registry = Arc::new(crate::payment_notify::PaymentNotifyRegistry::new());
    let conv_store = ConversationStore::from_env().await;
    let logistics = Arc::new(build_logistics_service());
    let mut handler_builder = MessageHandler::new(
        core,
        outbound.clone(),
        sessions,
        payment_registry,
        conv_store,
        logistics,
    );
    if let Some(pool) = build_wa_contacts_pool().await {
        handler_builder = handler_builder.with_pg_pool(pool);
    }
    let handler = Arc::new(handler_builder);

    maybe_spawn_tracking_monitor(outbound.clone()).await;
    let handler_health = handler.clone();

    tokio::spawn(async move {
        while let Some(ev) = rx.recv().await {
            if let Err(e) = handler.handle_event(ev).await {
                tracing::error!(error = %e, "message handler");
            }
        }
    });

    match cfg.transport {
        WaTransport::Cloud | WaTransport::Both => {
            let tx_webhook = tx.clone();
            let bind = cfg.webhook_bind.clone();
            let verify_token = cfg.verify_token.clone();
            tokio::spawn(async move {
                let wstate = WebhookState {
                    tx: tx_webhook,
                    verify_token,
                };
                if let Err(e) = run_webhook_server(&bind, wstate).await {
                    tracing::error!(error = %e, "webhook server");
                }
            });
        }
        WaTransport::Rust => {
            let bind = cfg.webhook_bind.clone();
            tokio::spawn(async move {
                if let Err(e) = run_health_only_server(&bind, handler_health).await {
                    tracing::error!(error = %e, "health server");
                }
            });
        }
    }

    Ok(tx)
}

fn build_logistics_service() -> LogisticsService {
    match LogisticsService::from_env() {
        Ok(svc) => {
            tracing::info!("whatsapp: logistics configurado (MELHOR_ENVIO_TOKEN presente)");
            svc
        }
        Err(_) => {
            tracing::warn!("whatsapp: MELHOR_ENVIO_TOKEN ausente — rastreio via Correios/LinkTrack se configurados");
            let token = "MISSING_TOKEN".to_string();
            LogisticsService::new(MelhorEnvioClient::new(token, true))
        }
    }
}

fn build_cascading_tracker() -> CascadingTracker {
    let sandbox = std::env::var("MELHOR_ENVIO_SANDBOX").map(|v| v == "1").unwrap_or(true);
    let token = std::env::var("MELHOR_ENVIO_TOKEN").unwrap_or_else(|_| "MISSING_TOKEN".to_string());
    CascadingTracker::from_env(MelhorEnvioClient::new(token, sandbox))
}

async fn maybe_spawn_tracking_monitor(outbound: Arc<Outbound>) {
    let db_url = match std::env::var("DATABASE_URL").ok().filter(|s| !s.trim().is_empty()) {
        Some(u) => u,
        None => return,
    };
    match sqlx::postgres::PgPoolOptions::new()
        .max_connections(3)
        .connect(db_url.trim())
        .await
    {
        Ok(pool) => {
            let tracker = build_cascading_tracker();
            let monitor = TrackingMonitor::new(pool, tracker, outbound);
            monitor.spawn();
            tracing::info!("tracking_monitor: iniciado (DATABASE_URL configurado)");
        }
        Err(e) => {
            tracing::warn!(error = %e, "tracking_monitor: falha ao conectar ao Postgres, monitor desativado");
        }
    }
}

/// Pool dedicado para wa_contacts (reutiliza DATABASE_URL se disponível).
async fn build_wa_contacts_pool() -> Option<sqlx::PgPool> {
    let db_url = std::env::var("DATABASE_URL").ok().filter(|s| !s.trim().is_empty())?;
    match sqlx::postgres::PgPoolOptions::new()
        .max_connections(3)
        .connect(db_url.trim())
        .await
    {
        Ok(pool) => {
            tracing::info!("wa_contacts: pool Postgres iniciado");
            Some(pool)
        }
        Err(e) => {
            tracing::warn!(error = %e, "wa_contacts: falha ao conectar Postgres, contatos não serão persistidos");
            None
        }
    }
}
