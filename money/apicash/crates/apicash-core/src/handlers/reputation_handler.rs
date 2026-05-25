//! Reputation and trust-seal handlers.

use std::sync::Arc;

use apicash_auth::JwtClaims;
use axum::extract::{Path, Query, State};
use axum::Extension;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ReputationQuery {
    pub completed: Option<u32>,
    pub dispute_count: Option<u32>,
    pub kyc_approved: Option<bool>,
}

/// `GET /reputation/:user_id` — compute reputation for any user (admin or self).
///
/// Callers provide `completed`, `dispute_count`, and `kyc_approved` as query params
/// until a dedicated profile service centralises that data.
pub async fn get_reputation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<ReputationQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if claims.sub != user_id && claims.role != apicash_auth::Role::Admin {
        return Err(ApiError::unauthorized("access denied"));
    }

    let completed = q.completed.unwrap_or(0);
    let dispute_count = q.dispute_count.unwrap_or(0);
    let kyc_approved = q.kyc_approved.unwrap_or(false);

    let rep = state
        .reputation
        .compute(user_id, completed, dispute_count, kyc_approved)
        .await
        .map_err(|e| ApiError::internal(e))?;

    Ok(Json(serde_json::json!({
        "user_id": rep.user_id,
        "score": rep.score,
        "completed_transactions": rep.completed_transactions,
        "dispute_rate": rep.dispute_rate,
        "seal": rep.seal.as_ref().map(|s| serde_json::json!({
            "name": s,
            "label": s.label(),
            "badge_color": s.badge_color(),
        })),
        "kyc_approved": rep.kyc_approved,
        "computed_at": rep.computed_at,
    })))
}
