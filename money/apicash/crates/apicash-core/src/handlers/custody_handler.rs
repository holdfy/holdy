//! Custody release HTTP handler.

use std::sync::Arc;

use apicash_auth::JwtClaims;
use apicash_custody::ReleaseConfirmation;
use apicash_shared::OrderStatus;
use apicash_shared::{ApiCashError, AuditEvent};
use axum::extract::State;
use axum::Extension;
use axum::Json;
use chrono::Utc;
use tracing::{error, info, instrument, warn};

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

    let stored_order = state
        .orders
        .get(body.order_id)
        .await
        .map_err(|e| {
            error!(error = %e, "order lookup failed");
            ApiError::internal("order lookup failed")
        })?
        .ok_or_else(|| {
            let ts = Utc::now();
            let ev = AuditEvent::DeliveryConfirmed {
                user_id: releasing_user_id,
                order_id: body.order_id,
                success: false,
                timestamp: ts,
            };
            warn!(
                user_id = %releasing_user_id,
                order_id = %body.order_id,
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
            order_id: Some(body.order_id),
            action: "DeliveryConfirmed".into(),
            reason: "user is not buyer for order".into(),
            timestamp: ts,
        };
        warn!(
            user_id = %releasing_user_id,
            order_id = %body.order_id,
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
        idempotency_key: body.idempotency_key,
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
            order_id: body.order_id,
            success: true,
            timestamp: ts,
        };
        info!(
            user_id = %releasing_user_id,
            order_id = %body.order_id,
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
            order_id: body.order_id,
            success: true,
            timestamp: ts,
        };
        info!(
            user_id = %releasing_user_id,
            order_id = %body.order_id,
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
    state.orders.update(stored_order).await.map_err(|e| {
        error!(error = %e, "order completion persistence failed");
        ApiError::internal("order completion persistence failed")
    })?;
    info!(order_id = %body.order_id, "order marked completed after custody release");

    Ok(Json(serde_json::json!({
        "custody_id": result.custody_id,
        "order_id": result.order_id,
        "yield_distributed": result.yield_distributed,
    })))
}
