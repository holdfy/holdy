use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;

use super::service::{AnchorService, ServiceError};
use super::types::{AuditItem, TransactionAnchorRow};

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<dyn AnchorService>,
    pub explorer_base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub entity_type: Option<String>,
    pub period_type: Option<String>,
    pub period_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub fn routes(service: Arc<dyn AnchorService>) -> Router {
    let explorer_base_url = std::env::var("EXPLORER_BASE_URL").unwrap_or_else(|_| "https://polygonscan.com".to_string());
    Router::new()
        .route("/audit", get(audit))
        .with_state(AppState {
            service,
            explorer_base_url,
        })
}

fn parse_optional_datetime(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn to_audit_item(row: &TransactionAnchorRow, explorer_base: &str) -> AuditItem {
    let explorer_url = row
        .tx_hash
        .as_ref()
        .filter(|h| !h.is_empty())
        .map(|tx_hash| format!("{}/tx/{}", explorer_base.trim_end_matches('/'), tx_hash));
    AuditItem {
        id: row.id,
        idempotency_key: row.idempotency_key.clone(),
        entity_type: row.entity_type.clone(),
        entity_id: row.entity_id.clone(),
        payload_hash: row.payload_hash.clone(),
        tx_hash: row.tx_hash.clone(),
        block_number: row.block_number,
        chain_id: row.chain_id,
        anchored_at: row.anchored_at,
        dry_run: row.dry_run,
        explorer_url,
        created_at: row.created_at,
    }
}

async fn audit(
    State(state): State<AppState>,
    Query(q): Query<AuditQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let from = q
        .from
        .as_deref()
        .and_then(parse_optional_datetime);
    let to = q.to.as_deref().and_then(parse_optional_datetime);
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);

    let (rows, total) = state
        .service
        .list_audit(
            from,
            to,
            q.entity_type,
            q.period_type,
            q.period_id,
            limit,
            offset,
        )
        .await?;

    let items: Vec<AuditItem> = rows
        .iter()
        .map(|r| to_audit_item(r, &state.explorer_base_url))
        .collect();

    Ok(Json(serde_json::json!({
        "items": items,
        "total": total
    })))
}

#[derive(Debug)]
pub enum AppError {
    Service(ServiceError),
}
impl From<ServiceError> for AppError {
    fn from(e: ServiceError) -> Self {
        AppError::Service(e)
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::Service(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        (status, Json(serde_json::json!({ "message": message }))).into_response()
    }
}
