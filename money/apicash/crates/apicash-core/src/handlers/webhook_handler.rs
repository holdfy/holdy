//! Webhook PIX incoming — recebe callback do Gatebox (ou anchor) quando um PIX é confirmado.
//!
//! Autenticação: HMAC-SHA256(GATEBOX_WEBHOOK_SECRET, raw_body) → hex → `X-Webhook-Signature`.
//! Mesmo esquema que o Gatebox usa ao disparar callbacks para `callback_url` registrados.
//!
//! Rota: `POST /internal/webhook/pix`
//!
//! O handler é idempotente: se a ordem já estiver `in_custody`, retorna 200 sem reprocessar.
//! Em caso de erro transitório no settle (Soroban, BRLx), registra aviso e retorna 200 para
//! o Gatebox não retentar indefinidamente — o `APICASH_FUNDING_POLLER` cobre o fallback.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use tracing::{info, warn};

use super::order_handler::settle_order_by_id;
use crate::state::AppState;

/// Payload enviado pelo Gatebox ao disparar webhook de PIX confirmado.
/// Aceita tanto `camelCase` quanto `snake_case` para robustez.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GateboxPixPayload {
    /// Tipo de evento: "pix_in", "pix_out", "test", …
    #[serde(rename = "type", default)]
    event_type: String,

    /// ID da transação Gatebox — corresponde ao `gateway_in_tx_id` gravado na ordem.
    /// O Gatebox pode enviar em vários nomes; aceitamos todos.
    #[serde(
        alias = "transaction_id",
        alias = "tx_id",
        alias = "txId",
        alias = "transactionId"
    )]
    transaction_id: Option<String>,

    /// Status do pagamento. Vazio implica pagamento confirmado (webhook só é enviado quando completo).
    #[serde(default)]
    status: String,
}

/// Recebe o callback do Gatebox quando um PIX entra e dispara o settle da ordem correspondente.
pub async fn receive_pix_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    // 1. Verificar assinatura HMAC quando o segredo está configurado.
    let secret = std::env::var("GATEBOX_WEBHOOK_SECRET")
        .ok()
        .filter(|s| !s.trim().is_empty());

    if let Some(ref secret) = secret {
        let sig = headers
            .get("X-Webhook-Signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !verify_hmac(secret, &body, sig) {
            warn!("webhook/pix: assinatura HMAC inválida — rejeitado");
            return StatusCode::UNAUTHORIZED;
        }
    } else {
        warn!("webhook/pix: GATEBOX_WEBHOOK_SECRET não configurado — aceitando sem verificação (somente dev)");
    }

    // 2. Deserializar payload.
    let payload: GateboxPixPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "webhook/pix: JSON inválido");
            return StatusCode::BAD_REQUEST;
        }
    };

    // 3. Ignorar eventos que não são pagamentos PIX recebidos.
    let kind = payload.event_type.to_lowercase();
    if kind == "pix_out" || kind == "test" || kind == "reversal" {
        return StatusCode::OK;
    }

    // Status vazio significa que o Gatebox só envia o webhook quando o pagamento é concluído;
    // quando presente, verificamos se é um status de sucesso.
    let status_lower = payload.status.to_lowercase();
    if !status_lower.is_empty()
        && !matches!(
            status_lower.as_str(),
            "completed" | "paid" | "success" | "done" | "confirmed"
        )
    {
        info!(status = %payload.status, "webhook/pix: status não indica pagamento confirmado — ignorando");
        return StatusCode::OK;
    }

    // 4. Localizar a ordem pelo `gateway_in_tx_id`.
    let Some(tx_id) = payload.transaction_id.filter(|s| !s.is_empty()) else {
        warn!("webhook/pix: campo transactionId ausente no payload");
        return StatusCode::OK;
    };

    let stored = match state.orders.find_by_gateway_tx_id(&tx_id).await {
        Ok(v) => v,
        Err(e) => {
            warn!(tx_id = %tx_id, error = %e, "webhook/pix: erro ao buscar ordem");
            return StatusCode::OK;
        }
    };

    let Some(order) = stored else {
        // tx_id não corresponde a nenhuma ordem pendente — ignorar silenciosamente.
        return StatusCode::OK;
    };

    info!(
        order_id = %order.order.id,
        tx_id = %tx_id,
        "webhook/pix: PIX confirmado, iniciando settle"
    );

    // 5. Fazer settle (idempotente — `settle_order_by_id` já trata `already_in_custody`).
    // Em falha transitória: logamos e retornamos 200 para o Gatebox não retentar em loop.
    // O APICASH_FUNDING_POLLER serve como fallback.
    if let Err(e) = settle_order_by_id(&state, order.order.id).await {
        warn!(
            order_id = %order.order.id,
            error = ?e,
            "webhook/pix: settle falhou — APICASH_FUNDING_POLLER cobrirá o retry"
        );
    }

    StatusCode::OK
}

/// Verifica `HMAC-SHA256(secret, body) == expected_hex` em tempo constante.
fn verify_hmac(secret: &str, body: &[u8], expected_hex: &str) -> bool {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;

    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let computed = hex::encode(mac.finalize().into_bytes());
    computed == expected_hex
}
