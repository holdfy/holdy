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
    state
        .disputes
        .resolve_dispute(id, body.resolution, body.notes)
        .await?;
    Ok(Json(serde_json::json!({ "ok": true, "dispute_id": id })))
}
