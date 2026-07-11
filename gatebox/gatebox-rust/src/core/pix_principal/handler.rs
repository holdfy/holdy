// Stub handler and register from app/modules/core/pix_principal/handler
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::core::pix_principal::blindpay_webhook::{self, BlindPayWebhookState};
use crate::core::pix_principal::service::{GenerateQrCodeRequest, GenerateQrCodeResponse, PixPrincipalService, SendPixRequest, SendPixResponse};
use crate::core::pix_principal::webhook_service::{
    PixWebhookService, ReceivePixInRequest, ReceivePixInResponse, ReceivePixOutRequest,
    ReceivePixOutResponse,
};
use crate::bank_bridge::QrRefCache;

#[derive(Clone)]
pub struct PixPrincipalState {
    pub service: std::sync::Arc<dyn PixPrincipalService>,
    pub webhook_service: Option<std::sync::Arc<dyn PixWebhookService>>,
    pub qr_cache: QrRefCache,
}

pub fn register(state: PixPrincipalState, blindpay_webhook_state: Option<BlindPayWebhookState>) -> Router {
    let mut r = Router::new()
        .route("/send", post(send_pix))
        .route("/qrcode", post(generate_qrcode));
    if state.webhook_service.is_some() {
        r = r
            .route("/webhook/in", post(receive_pix_in))
            .route("/webhook/out", post(receive_pix_out));
    }
    let r = r.with_state(state);

    match blindpay_webhook_state {
        Some(bp_state) => {
            let bp_router = Router::new()
                .route("/webhook/blindpay", post(blindpay_webhook::receive_blindpay_webhook))
                .with_state(bp_state);
            r.merge(bp_router)
        }
        None => r,
    }
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
    let reference = body.reference.clone().unwrap_or_default();
    let req = GenerateQrCodeRequest {
        amount: body.amount,
        payer_name: body.payer_name.unwrap_or_default(),
        payer_document: body.payer_document.unwrap_or_default(),
        description: body.description.unwrap_or_default(),
        expiration_seconds: body.expiration_seconds.unwrap_or(1800),
        reference: reference.clone(),
        pix_key: body.pix_key,
    };
    let resp = state
        .service
        .generate_qr_code(req)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Se o QR retornado é um BR Code EMV real (Sulcred), armazena emv_hash → reference
    // para que notify_status possa resolver o order:uuid original.
    if !reference.is_empty() && resp.qr_code.starts_with("000201") {
        let emv_hash = emv_stub_charge_id(&resp.qr_code);
        if let Ok(mut cache) = state.qr_cache.write() {
            cache.insert(emv_hash, reference);
        }
    }

    Ok(Json(resp))
}

fn emv_stub_charge_id(reference: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(reference.as_bytes());
    format!("sandbox-emv-{}", hex::encode(&h.finalize()[..10]))
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
