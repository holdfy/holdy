// Backoffice HTTP handlers (from app/modules/backoffice/handler/*)
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::accounts::AccountsService;
use crate::modules::shared::auth::{create_token, BackofficeAuth};

use crate::app_log::AppLogRepository;

#[derive(Clone)]
pub struct BackofficeState {
    pub accounts_svc: Arc<dyn AccountsService>,
    pub app_log_repo: Option<Arc<dyn AppLogRepository>>,
}

// ---- Auth ----
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: LoginUser,
}

#[derive(serde::Serialize)]
pub struct LoginUser {
    pub id: i32,
    pub username: String,
    pub role: String,
}

async fn auth_login(Json(req): Json<LoginRequest>) -> Result<Json<LoginResponse>, AppError> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest("Username and password are required"));
    }
    let token = create_token(1, &req.username, "backoffice").map_err(|_| AppError::Internal)?;
    Ok(Json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 86400,
        user: LoginUser {
            id: 1,
            username: req.username,
            role: "backoffice".to_string(),
        },
    }))
}

async fn auth_profile(BackofficeAuth(auth): BackofficeAuth) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": auth.user_id,
        "username": auth.username,
        "role": auth.role,
    }))
}

// ---- Logs (stubs) ----
#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub level: Option<String>,
    pub service: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

async fn logs_list(
    State(state): State<BackofficeState>,
    _auth: BackofficeAuth,
    Query(q): Query<LogsQuery>,
) -> Json<serde_json::Value> {
    let limit = q.limit.unwrap_or(50).clamp(1, 100);
    let page = q.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;
    let (data, total) = match &state.app_log_repo {
        Some(repo) => {
            let level = q.level.as_deref();
            let service = q.service.as_deref();
            match repo.list(level, service, offset, limit).await {
                Ok(rows) => {
                    let total = rows.len() as i64;
                    (serde_json::to_value(&rows).unwrap_or(serde_json::json!([])), total)
                }
                Err(_) => (serde_json::json!([]), 0),
            }
        }
        None => (serde_json::json!([]), 0),
    };
    Json(serde_json::json!({
        "data": data,
        "pagination": { "page": page, "limit": limit, "total": total },
    }))
}

async fn logs_metrics(_auth: BackofficeAuth) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "requests_per_second": 0,
        "avg_response_time_ms": 0,
        "error_rate": 0,
    }))
}

async fn logs_transactions(_auth: BackofficeAuth) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "data": [] }))
}

async fn logs_errors(_auth: BackofficeAuth) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "data": [] }))
}

// ---- Accounts (delegate to accounts_svc) ----
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    #[allow(dead_code)]
    pub status: Option<String>,
}

async fn accounts_list(
    State(state): State<BackofficeState>,
    _auth: BackofficeAuth,
    Query(p): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = p.limit.unwrap_or(20).clamp(1, 100);
    let page = p.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;
    let page_result = state.accounts_svc.list(offset, limit).await?;
    Ok(Json(serde_json::json!({
        "data": page_result.items,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": page_result.total,
        },
    })))
}

async fn accounts_statistics(_auth: BackofficeAuth) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "total_accounts": 0,
        "active_accounts": 0,
        "pending_accounts": 0,
    }))
}

async fn accounts_get(
    State(state): State<BackofficeState>,
    _auth: BackofficeAuth,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let account = state.accounts_svc.get_by_id(id).await?;
    let a = account.ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::to_value(a).map_err(|_| AppError::Internal)?))
}

async fn accounts_transactions(
    _auth: BackofficeAuth,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "data": [] }))
}

async fn accounts_update_status(
    State(state): State<BackofficeState>,
    _auth: BackofficeAuth,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let status = body
        .get("status")
        .and_then(|v| v.as_i64())
        .ok_or(AppError::BadRequest("status required"))?;
    let mut account = state
        .accounts_svc
        .get_by_id(id)
        .await?
        .ok_or(AppError::NotFound)?;
    account.account_status_id = status;
    state.accounts_svc.update(id, &account).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug)]
pub enum AppError {
    BadRequest(&'static str),
    NotFound,
    Internal,
    Accounts(crate::accounts::AccountsHandlerAppError),
}

impl From<crate::accounts::ServiceError> for AppError {
    fn from(e: crate::accounts::ServiceError) -> Self {
        AppError::Accounts(crate::accounts::AccountsHandlerAppError::Service(e))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
            AppError::Accounts(e) => return e.into_response(),
        };
        (status, Json(serde_json::json!({ "error": msg }))).into_response()
    }
}

pub fn routes(state: BackofficeState) -> Router {
    Router::new()
        .route("/auth/login", post(auth_login))
        .route("/auth/profile", get(auth_profile))
        .route("/logs", get(logs_list))
        .route("/logs/metrics", get(logs_metrics))
        .route("/logs/transactions", get(logs_transactions))
        .route("/logs/errors", get(logs_errors))
        .route("/accounts", get(accounts_list))
        .route("/accounts/statistics", get(accounts_statistics))
        .route("/accounts/:id", get(accounts_get))
        .route("/accounts/:id/transactions", get(accounts_transactions))
        .route("/accounts/:id/status", put(accounts_update_status))
        .with_state(state)
}
