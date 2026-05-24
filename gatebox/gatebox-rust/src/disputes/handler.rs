use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::modules::shared::auth::AdminAuth;
use super::model::{CreateDisputeRequest, ResolveDisputeRequest};
use super::repository::DisputeRepository;

#[derive(Clone)]
pub struct DisputeState {
    pub repo: Arc<dyn DisputeRepository>,
}


#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub account_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

async fn list_disputes(
    State(state): State<DisputeState>,
    _auth: AdminAuth,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let offset = q.offset.unwrap_or(0).max(0);

    let page = if let Some(account_id) = q.account_id {
        state.repo.list_by_account(account_id, offset, limit).await?
    } else {
        state.repo.list(q.status.as_deref(), offset, limit).await?
    };

    Ok(Json(serde_json::json!({
        "items": page.items,
        "total": page.total,
        "limit": limit,
        "offset": offset,
    })))
}

async fn create_dispute(
    State(state): State<DisputeState>,
    _auth: AdminAuth,
    Json(req): Json<CreateDisputeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if req.reason.trim().is_empty() {
        return Err(AppError::BadRequest("reason is required"));
    }
    let dispute_type = req.r#type.as_deref().unwrap_or("INFRACTION");
    let id = state.repo
        .insert(req.transaction_id, req.account_id, dispute_type, &req.reason, req.evidence.as_ref())
        .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

async fn get_dispute(
    State(state): State<DisputeState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let dispute = state.repo.get_by_id(id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::to_value(dispute).map_err(|_| AppError::Internal)?))
}

async fn resolve_dispute(
    State(state): State<DisputeState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
    Json(req): Json<ResolveDisputeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let notes = req.notes.as_deref();
    let resolved = state.repo.resolve(id, req.resolved_by, notes).await?;
    if !resolved {
        return Err(AppError::BadRequest("Dispute not found or already resolved"));
    }
    tracing::info!(dispute_id = id, resolution = %req.resolution, "dispute resolved");
    Ok(Json(serde_json::json!({
        "message": "Dispute resolved",
        "id": id,
        "resolution": req.resolution,
    })))
}

/// Customer-facing: list own disputes (no AdminAuth required — uses CustomerAuth or account_id param).
async fn list_customer_disputes(
    State(state): State<DisputeState>,
    Path(account_id): Path<i64>,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let offset = q.offset.unwrap_or(0).max(0);
    let page = state.repo.list_by_account(account_id, offset, limit).await?;
    Ok(Json(serde_json::json!({
        "items": page.items,
        "total": page.total,
    })))
}

#[derive(Debug)]
pub enum AppError {
    BadRequest(&'static str),
    NotFound,
    Internal,
    Repository(super::repository::RepositoryError),
}

impl From<super::repository::RepositoryError> for AppError {
    fn from(e: super::repository::RepositoryError) -> Self {
        AppError::Repository(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
            AppError::Repository(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        (status, Json(serde_json::json!({ "error": msg }))).into_response()
    }
}

pub fn admin_routes(state: DisputeState) -> Router {
    Router::new()
        .route("/", get(list_disputes).post(create_dispute))
        .route("/:id", get(get_dispute))
        .route("/:id/resolve", post(resolve_dispute))
        .with_state(state)
}

pub fn customer_routes(state: DisputeState) -> Router {
    Router::new()
        .route("/accounts/:account_id/disputes", get(list_customer_disputes))
        .with_state(state)
}
