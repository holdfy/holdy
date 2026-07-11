//! Recebe o webhook nativo da BlindPay (payin/payout completo) e credita a transação PIX
//! correspondente, gravando `gateway_tx_id` (id do payin na BlindPay) e `chain_tx_hash`
//! (hash real da liquidação on-chain) — ver PLANO_EXECUCAO / migration
//! `20260710000000_transaction_gateway_tx_fields.sql`.
//!
//! Esquema de assinatura ainda não confirmado contra uma entrega real da BlindPay (docs
//! renderizadas em JS); segue o padrão Svix (`svix-id`/`svix-timestamp`/`svix-signature`,
//! `HMAC-SHA256(base64_decode(secret sem prefixo "whsec_"), "{id}.{timestamp}.{body}")`,
//! base64). Em caso de falha de verificação ou parse, os headers e o corpo cru são logados
//! em nível `warn` para permitir ajuste rápido assim que o primeiro evento real chegar.

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use tracing::warn;

use super::webhook_service::{PixWebhookService, ReceivePixInRequest};
use crate::transaction::TransactionRepository;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct BlindPayWebhookState {
    pub webhook_service: Arc<dyn PixWebhookService>,
    pub transaction_repo: Arc<dyn TransactionRepository>,
    pub webhook_secret: Option<String>,
    /// PIX key to credit — mesma usada na criação do QR (hoje sempre a fixa da plataforma).
    pub credit_pix_key: String,
}

#[derive(Debug, Default, Deserialize)]
struct TrackingStep {
    #[serde(default)]
    step: String,
    #[serde(default)]
    transaction_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BlindPayWebhookEvent {
    #[serde(default)]
    id: String,
    #[serde(default)]
    sender_amount: Option<i64>,
    #[serde(default)]
    first_name: String,
    #[serde(default)]
    last_name: String,
    #[serde(default)]
    tracking_payment: Option<TrackingStep>,
    #[serde(default)]
    tracking_complete: Option<TrackingStep>,
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn compute_signature(secret: &str, signed_content: &str) -> Option<String> {
    let raw_secret_b64 = secret.strip_prefix("whsec_").unwrap_or(secret);
    let key_bytes = STANDARD.decode(raw_secret_b64).ok()?;
    let mut mac = HmacSha256::new_from_slice(&key_bytes).ok()?;
    mac.update(signed_content.as_bytes());
    Some(STANDARD.encode(mac.finalize().into_bytes()))
}

/// `signature_header` pode conter múltiplos valores `v1,<base64>` separados por espaço
/// (esquema Svix, usado por várias plataformas de pagamento).
fn verify_signature(secret: &str, id: &str, timestamp: &str, body: &[u8], signature_header: &str) -> bool {
    let signed_content = format!("{id}.{timestamp}.{}", String::from_utf8_lossy(body));
    let Some(expected) = compute_signature(secret, &signed_content) else {
        return false;
    };
    signature_header
        .split_whitespace()
        .filter_map(|tok| tok.split_once(','))
        .any(|(_version, sig)| constant_time_eq(sig.as_bytes(), expected.as_bytes()))
}

pub async fn receive_blindpay_webhook(
    State(state): State<BlindPayWebhookState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    if let Some(secret) = state.webhook_secret.as_deref() {
        let id = headers.get("svix-id").and_then(|v| v.to_str().ok()).unwrap_or("");
        let timestamp = headers.get("svix-timestamp").and_then(|v| v.to_str().ok()).unwrap_or("");
        let signature = headers.get("svix-signature").and_then(|v| v.to_str().ok()).unwrap_or("");
        if id.is_empty() || timestamp.is_empty() || signature.is_empty()
            || !verify_signature(secret, id, timestamp, &body, signature)
        {
            warn!(
                ?headers,
                body = %String::from_utf8_lossy(&body),
                "blindpay webhook: assinatura inválida ou headers ausentes — rejeitado"
            );
            return StatusCode::UNAUTHORIZED;
        }
    } else {
        warn!("blindpay webhook: BLINDPAY_WEBHOOK_SECRET não configurado — aceitando sem verificação (somente dev)");
    }

    let event: BlindPayWebhookEvent = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(e) => {
            warn!(error = %e, body = %String::from_utf8_lossy(&body), "blindpay webhook: JSON inválido");
            return StatusCode::BAD_REQUEST;
        }
    };

    if event.id.is_empty() {
        warn!("blindpay webhook: evento sem id (payin id) — ignorando");
        return StatusCode::OK;
    }

    let payment_completed = event
        .tracking_payment
        .as_ref()
        .map(|t| t.step == "completed")
        .unwrap_or(false);

    if payment_completed {
        let amount = event.sender_amount.unwrap_or(0) as f64 / 100.0;
        let payer_name = format!("{} {}", event.first_name, event.last_name).trim().to_string();
        let req = ReceivePixInRequest {
            end_to_end_id: event.id.clone(),
            amount,
            pix_key: state.credit_pix_key.clone(),
            is_qr_code_payment: true,
            payer_name,
            payer_document: String::new(),
            // Precisa conter "holdfy" — é o que SQL_LIST_HOLDFY usa pra filtrar a tela
            // "Transações HoldFy" do admin, já que o external_id aqui é o payin id da
            // BlindPay (pi_...), não bate com o padrão "order%".
            description: format!("HoldFy PIX via BlindPay ({})", event.id),
            idempotency_key: event.id.clone(),
            gateway: "blindpay".to_string(),
            gateway_tx_id: event.id.clone(),
        };
        if let Err(e) = state.webhook_service.receive_pix_in(req).await {
            warn!(error = %e, payin_id = %event.id, "blindpay webhook: falha ao creditar PIX IN");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    if let Some(hash) = event.tracking_complete.as_ref().and_then(|t| t.transaction_hash.as_deref()) {
        if !hash.is_empty() {
            if let Err(e) = state
                .transaction_repo
                .update_chain_tx_hash_by_gateway_tx_id(&event.id, hash)
                .await
            {
                warn!(error = %e, payin_id = %event.id, "blindpay webhook: falha ao gravar chain_tx_hash");
            }
        }
    }

    StatusCode::OK
}
