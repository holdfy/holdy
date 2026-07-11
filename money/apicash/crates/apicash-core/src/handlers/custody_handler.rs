//! Custody release HTTP handler.

use std::sync::Arc;

use apicash_auth::JwtClaims;
use apicash_custody::ReleaseConfirmation;
use apicash_shared::OrderStatus;
use apicash_shared::{ApiCashError, AuditEvent};
use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::Extension;
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::dto::ReleaseRequestBody;
use crate::error::ApiError;
use crate::state::AppState;

#[instrument(skip(state, body), fields(order_id = %body.order_id))]
pub async fn release_custody(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<JwtClaims>>,
    Json(body): Json<ReleaseRequestBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if body.idempotency_key.trim().is_empty() {
        return Err(ApiError::bad_request("idempotency_key is required"));
    }

    // Security/business rule (critical): **only the buyer** may confirm delivery and release funds.
    // In production, the releasing identity must come from JWT (`sub`).
    let releasing_user_id = if state.auth.config().auth_disabled {
        body.released_by
    } else {
        let Some(Extension(c)) = claims else {
            let ts = Utc::now();
            let ev = AuditEvent::UnauthorizedAttempt {
                user_id: None,
                order_id: Some(body.order_id),
                action: "DeliveryConfirmed".into(),
                reason: "missing JWT".into(),
                timestamp: ts,
            };
            warn!(
                order_id = %body.order_id,
                action = "DeliveryConfirmed",
                success = false,
                timestamp = %ts,
                audit = ?ev,
                "audit"
            );
            return Err(ApiError::unauthorized("missing JWT"));
        };
        if c.sub != body.released_by {
            let ts = Utc::now();
            let ev = AuditEvent::UnauthorizedAttempt {
                user_id: Some(c.sub),
                order_id: Some(body.order_id),
                action: "DeliveryConfirmed".into(),
                reason: "released_by mismatch".into(),
                timestamp: ts,
            };
            warn!(
                user_id = %c.sub,
                order_id = %body.order_id,
                action = "DeliveryConfirmed",
                success = false,
                timestamp = %ts,
                audit = ?ev,
                "audit"
            );
            return Err(ApiError::unauthorized(
                "released_by must match the authenticated user",
            ));
        }
        c.sub
    };

    finalize_release(&state, body.order_id, releasing_user_id, body.idempotency_key).await
}

/// Lógica compartilhada entre `release_custody` (JWT, comprador real) e
/// `release_custody_internal` (dev/testnet, força liberação sem comprador logado).
/// O caller já validou que `releasing_user_id` pode agir como comprador do pedido.
async fn finalize_release(
    state: &Arc<AppState>,
    order_id: Uuid,
    releasing_user_id: Uuid,
    idempotency_key: String,
) -> Result<Json<serde_json::Value>, ApiError> {
    let stored_order = state
        .orders
        .get(order_id)
        .await
        .map_err(|e| {
            error!(error = %e, "order lookup failed");
            ApiError::internal("order lookup failed")
        })?
        .ok_or_else(|| {
            let ts = Utc::now();
            let ev = AuditEvent::DeliveryConfirmed {
                user_id: releasing_user_id,
                order_id,
                success: false,
                timestamp: ts,
            };
            warn!(
                user_id = %releasing_user_id,
                order_id = %order_id,
                action = "DeliveryConfirmed",
                success = false,
                timestamp = %ts,
                audit = ?ev,
                "audit"
            );
            ApiError::not_found("order not found")
        })?;
    let order = stored_order.order.clone();

    if !order.is_buyer(&releasing_user_id) {
        let ts = Utc::now();
        let ev = AuditEvent::UnauthorizedAttempt {
            user_id: Some(releasing_user_id),
            order_id: Some(order_id),
            action: "DeliveryConfirmed".into(),
            reason: "user is not buyer for order".into(),
            timestamp: ts,
        };
        warn!(
            user_id = %releasing_user_id,
            order_id = %order_id,
            action = "DeliveryConfirmed",
            success = false,
            timestamp = %ts,
            audit = ?ev,
            "audit"
        );
        return Err(ApiError::from(ApiCashError::UnauthorizedRelease));
    }

    let confirmation = ReleaseConfirmation {
        released_by: releasing_user_id,
        idempotency_key,
    };

    let result = state
        .custody
        .release_funds(&order, releasing_user_id, confirmation)
        .await
        .map_err(|e| {
            error!(error = %e, "custody release failed");
            ApiError::from(e)
        })?;
    {
        let ts = Utc::now();
        let ev = AuditEvent::DeliveryConfirmed {
            user_id: releasing_user_id,
            order_id,
            success: true,
            timestamp: ts,
        };
        info!(
            user_id = %releasing_user_id,
            order_id = %order_id,
            action = "DeliveryConfirmed",
            success = true,
            timestamp = %ts,
            audit = ?ev,
            "audit"
        );
    }
    {
        let ts = Utc::now();
        let ev = AuditEvent::FundsReleased {
            user_id: releasing_user_id,
            order_id,
            success: true,
            timestamp: ts,
        };
        info!(
            user_id = %releasing_user_id,
            order_id = %order_id,
            action = "FundsReleased",
            success = true,
            timestamp = %ts,
            audit = ?ev,
            "audit"
        );
    }

    let mut stored_order = stored_order;
    stored_order.order.status = OrderStatus::Completed;
    stored_order.order.updated_at = Utc::now();
    let mut completed_order = stored_order.clone();
    state.orders.update(stored_order).await.map_err(|e| {
        error!(error = %e, "order completion persistence failed");
        ApiError::internal("order completion persistence failed")
    })?;
    info!(order_id = %order_id, "order marked completed after custody release");

    // Auto off-ramp: se o vendedor registrou chave PIX, disparar transferência imediatamente.
    if completed_order.off_ramp_tx_hash.is_none() {
        if let Some(repo) = &state.listing_repo {
            if let Some(pix_key) = repo.pix_key_for_user(order.seller_id).await {
                match state
                    .anchor
                    .withdraw_to_pix(
                        order.amount,
                        pix_key.clone(),
                        format!("order:{}:offramp:auto", order_id),
                        format!("auto off-ramp order:{}", order_id),
                    )
                    .await
                {
                    Ok(resp) => {
                        completed_order.off_ramp_tx_hash = Some(resp.tx_hash.clone());
                        if let Err(e) = state.orders.update(completed_order).await {
                            warn!(order_id = %order_id, error = %e, "off-ramp hash persist failed");
                        }
                        info!(order_id = %order_id, tx = %resp.tx_hash, "auto off-ramp OK");
                    }
                    Err(e) => {
                        warn!(order_id = %order_id, error = %e, "auto off-ramp failed (non-critical — manual off-ramp available)");
                    }
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "custody_id": result.custody_id,
        "order_id": result.order_id,
        "yield_distributed": result.yield_distributed,
    })))
}

#[derive(Debug, Deserialize)]
pub struct InternalReleaseRequest {
    pub order_id: Uuid,
}

/// Dev/testnet only: força a liberação de custódia (equivalente ao comprador confirmar entrega)
/// sem exigir JWT — autenticado por `APICASH_API_KEY`, igual `settle_order_internal`.
#[instrument(skip(state, headers), fields(order_id = %req.order_id))]
pub async fn release_custody_internal(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<InternalReleaseRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let expected = std::env::var("APICASH_API_KEY").unwrap_or_default();
    if expected.is_empty() {
        return Err(ApiError::internal("internal API key not configured"));
    }
    let got = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer ").or_else(|| h.strip_prefix("bearer ")))
        });
    if got != Some(expected.as_str()) {
        return Err(ApiError::unauthorized("missing or invalid internal API key"));
    }

    let stored_order = state
        .orders
        .get(req.order_id)
        .await
        .map_err(|e| ApiError::internal(format!("order lookup failed: {e}")))?
        .ok_or_else(|| ApiError::not_found("order not found"))?;

    if stored_order.order.status == OrderStatus::Completed {
        return Ok(Json(serde_json::json!({
            "order_id": req.order_id,
            "status": "already_completed"
        })));
    }
    if stored_order.order.status != OrderStatus::InCustody {
        return Err(ApiError::bad_request(
            "order must be in_custody to release",
        ));
    }

    let buyer_id = stored_order.order.buyer_id;
    let idempotency_key = format!("dev-force-release:{}", req.order_id);
    finalize_release(&state, req.order_id, buyer_id, idempotency_key).await
}
