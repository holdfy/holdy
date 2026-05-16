use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::model::AccountTypes;
use crate::shared::types::ItemsPage;
use super::service::{AccountTypesService, ServiceError};

#[derive(Clone)]
pub struct AppState { pub service: Arc<dyn AccountTypesService> }

#[derive(Debug, Deserialize)]
pub struct PaginationQuery { pub limit: Option<i64>, pub offset: Option<i64> }

pub fn routes(service: Arc<dyn AccountTypesService>) -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(get_by_id).put(update).delete(delete_by_id))
        .with_state(AppState { service })
}

async fn list(State(state): State<AppState>, Query(p): Query<PaginationQuery>) -> Result<Json<ItemsPage<Vec<AccountTypes>>>, AppError> {
    let limit = p.limit.unwrap_or(10).clamp(1, 100);
    let offset = p.offset.unwrap_or(0).max(0);
    let page = state.service.list(offset, limit).await?;
    Ok(Json(page))
}

async fn get_by_id(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<Option<AccountTypes>>, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let item = state.service.get_by_id(id).await?;
    Ok(Json(item))
}

async fn create(State(state): State<AppState>, Json(item): Json<AccountTypes>) -> Result<Json<serde_json::Value>, AppError> {
    let id = state.service.create(&item).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn update(State(state): State<AppState>, Path(id): Path<String>, Json(item): Json<AccountTypes>) -> Result<StatusCode, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    state.service.update(id, &item).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_by_id(State(state): State<AppState>, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let deleted = state.service.delete(id).await?;
    if !deleted { return Err(AppError::NotFound); }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug)]
pub enum AppError {
    Service(ServiceError),
    BadRequest(&'static str),
    NotFound,
}
impl From<ServiceError> for AppError { fn from(e: ServiceError) -> Self { AppError::Service(e) } }
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::Service(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, (*msg).to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
        };
        (status, Json(serde_json::json!({ "message": message }))).into_response()
    }
}
