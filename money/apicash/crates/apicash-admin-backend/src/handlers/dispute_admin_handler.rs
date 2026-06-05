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

    state
        .disputes
        .resolve_dispute(id, body.resolution, body.notes)
        .await?;

    // Notifica comprador e vendedor via WhatsApp (fire-and-forget).
    let verdict = match body.resolution {
        ResolutionType::RefundBuyer     => "favor_buyer",
        ResolutionType::ReleaseToSeller => "favor_seller",
        ResolutionType::Split           => "split",
        ResolutionType::Manual          => "manual",
    };
    notify_wa_dispute_resolved(order_id, verdict, &dispute.reason).await;

    Ok(Json(serde_json::json!({ "ok": true, "dispute_id": id, "verdict": verdict })))
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
