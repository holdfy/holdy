//! Disputas administrativas.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AdminError;
use crate::state::AdminState;
use apicash_disputes::{Dispute, ResolutionType};

#[derive(Debug, Deserialize)]
pub struct ResolveDisputeBody {
    pub resolution: ResolutionType,
    pub notes: Option<String>,
}

pub async fn list_disputes(
    State(state): State<AdminState>,
) -> Result<Json<Vec<Dispute>>, AdminError> {
    let list = state.disputes.list_all_disputes().await?;
    Ok(Json(list))
}

pub async fn get_dispute(
    State(state): State<AdminState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Dispute>, AdminError> {
    let d = state
        .disputes
        .get_dispute(id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("dispute {id}")))?;
    Ok(Json(d))
}

pub async fn resolve_dispute(
    State(state): State<AdminState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ResolveDisputeBody>,
) -> Result<Json<serde_json::Value>, AdminError> {
    // Busca dados antes de resolver (precisamos de order_id e amount para notificar WA).
    let dispute = state.disputes.get_dispute(id).await?
        .ok_or_else(|| AdminError::NotFound(format!("dispute {id}")))?;
    let order_id = dispute.order_id;

    // Busca order para obter o valor real (usado na mensagem WA).
    let amount_str = state.orders.get(order_id).await
        .ok()
        .flatten()
        .map(|s| s.order.amount.to_string())
        .unwrap_or_default();

    state
        .disputes
        .resolve_dispute(id, body.resolution, body.notes)
        .await?;

    let verdict = match body.resolution {
        ResolutionType::RefundBuyer     => "favor_buyer",
        ResolutionType::ReleaseToSeller => "favor_seller",
        ResolutionType::Split           => "split",
        ResolutionType::Manual          => "manual",
    };

    // Marca order Completed + dispara off-ramp PIX ao ganhador (fire-and-forget).
    if matches!(body.resolution, ResolutionType::RefundBuyer | ResolutionType::ReleaseToSeller) {
        let v = verdict.to_string();
        tokio::spawn(async move {
            finalize_dispute_order(order_id, &v, None).await;
        });
    }

    // Notifica comprador e vendedor via WhatsApp (fire-and-forget).
    notify_wa_dispute_resolved(order_id, verdict, &amount_str).await;

    Ok(Json(serde_json::json!({ "ok": true, "dispute_id": id, "verdict": verdict })))
}

/// Chama `POST /orders/{id}/dispute/complete` no apicash-core.
/// Marca order como Completed e dispara off-ramp PIX ao ganhador.
async fn finalize_dispute_order(order_id: Uuid, verdict: &str, pix_key: Option<&str>) {
    let core_url = std::env::var("APICASH_CORE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let api_key = std::env::var("APICASH_API_KEY").unwrap_or_default();
    let body = serde_json::json!({ "verdict": verdict, "pix_key": pix_key });
    let client = reqwest::Client::new();
    match client
        .post(format!("{core_url}/orders/{order_id}/dispute/complete"))
        .header("x-api-key", &api_key)
        .json(&body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() =>
            tracing::info!(%order_id, verdict, "dispute_complete: order finalized + off-ramp triggered"),
        Ok(r) =>
            tracing::warn!(%order_id, status = %r.status(), "dispute_complete: non-2xx"),
        Err(e) =>
            tracing::warn!(%order_id, error = %e, "dispute_complete: http failed"),
    }
}

/// Chama `POST /internal/dispute-resolved` no serviço WhatsApp (fire-and-forget).
async fn notify_wa_dispute_resolved(order_id: Uuid, verdict: &str, amount: &str) {
    let wa_url = std::env::var("APICASH_WA_INTERNAL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3010".to_string());
    let api_key = std::env::var("APICASH_API_KEY").unwrap_or_default();
    let body = serde_json::json!({
        "order_id": order_id,
        "verdict":  verdict,
        "amount":   amount,
    });
    let client = reqwest::Client::new();
    match client
        .post(format!("{wa_url}/internal/dispute-resolved"))
        .header("x-api-key", &api_key)
        .json(&body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() =>
            tracing::info!(%order_id, verdict, "WA dispute-resolved notified"),
        Ok(r) =>
            tracing::warn!(%order_id, status = %r.status(), "WA dispute-resolved non-2xx"),
        Err(e) =>
            tracing::warn!(%order_id, error = %e, "WA dispute-resolved call failed"),
    }
}
