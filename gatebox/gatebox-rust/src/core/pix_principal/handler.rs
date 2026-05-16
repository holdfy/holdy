// Stub handler and register from app/modules/core/pix_principal/handler
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::core::pix_principal::service::{GenerateQrCodeRequest, GenerateQrCodeResponse, PixPrincipalService, SendPixRequest, SendPixResponse};
use crate::core::pix_principal::webhook_service::{
    PixWebhookService, ReceivePixInRequest, ReceivePixInResponse, ReceivePixOutRequest,
    ReceivePixOutResponse,
};

#[derive(Clone)]
pub struct PixPrincipalState {
    pub service: std::sync::Arc<dyn PixPrincipalService>,
    pub webhook_service: Option<std::sync::Arc<dyn PixWebhookService>>,
}

pub fn register(state: PixPrincipalState) -> Router {
    let mut r = Router::new()
        .route("/send", post(send_pix))
        .route("/qrcode", post(generate_qrcode));
    if state.webhook_service.is_some() {
        r = r
            .route("/webhook/in", post(receive_pix_in))
            .route("/webhook/out", post(receive_pix_out));
    }
    r.with_state(state)
}

async fn send_pix(
    State(state): State<PixPrincipalState>,
    Json(req): Json<SendPixRequest>,
) -> Result<Json<SendPixResponse>, (axum::http::StatusCode, String)> {
    state
        .service
        .send_pix(req)
        .await
        .map(Json)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Deserialize)]
struct QrCodeBody {
    amount: f64,
    payer_name: Option<String>,
    payer_document: Option<String>,
    description: Option<String>,
    expiration_seconds: Option<i32>,
    reference: Option<String>,
    pix_key: Option<String>,
}

async fn generate_qrcode(
    State(state): State<PixPrincipalState>,
    Json(body): Json<QrCodeBody>,
) -> Result<Json<GenerateQrCodeResponse>, (axum::http::StatusCode, String)> {
    let req = GenerateQrCodeRequest {
        amount: body.amount,
        payer_name: body.payer_name.unwrap_or_default(),
        payer_document: body.payer_document.unwrap_or_default(),
        description: body.description.unwrap_or_default(),
        expiration_seconds: body.expiration_seconds.unwrap_or(1800),
        reference: body.reference.unwrap_or_default(),
        pix_key: body.pix_key,
    };
    state
        .service
        .generate_qr_code(req)
        .await
        .map(Json)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn receive_pix_in(
    State(state): State<PixPrincipalState>,
    Json(req): Json<ReceivePixInRequest>,
) -> Result<Json<ReceivePixInResponse>, (axum::http::StatusCode, String)> {
    let svc = state
        .webhook_service
        .as_ref()
        .ok_or_else(|| (axum::http::StatusCode::NOT_IMPLEMENTED, "webhook not configured".to_string()))?;
    svc.receive_pix_in(req)
        .await
        .map(Json)
        .map_err(|e| {
            let msg = e.to_string();
            let status = if msg.contains("not found") || msg.contains("is required") {
                axum::http::StatusCode::BAD_REQUEST
            } else {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, msg)
        })
}

async fn receive_pix_out(
    State(state): State<PixPrincipalState>,
    Json(req): Json<ReceivePixOutRequest>,
) -> Result<Json<ReceivePixOutResponse>, (axum::http::StatusCode, String)> {
    let svc = state
        .webhook_service
        .as_ref()
        .ok_or_else(|| (axum::http::StatusCode::NOT_IMPLEMENTED, "webhook not configured".to_string()))?;
    svc.receive_pix_out(req)
        .await
        .map(Json)
        .map_err(|e| {
            let msg = e.to_string();
            let status = if msg.contains("not found") || msg.contains("is required") {
                axum::http::StatusCode::BAD_REQUEST
            } else {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, msg)
        })
}
