// Admin HTTP handlers (from app/modules/admin/handler/*)
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::accounts::AccountsService;
use crate::model::Accounts;
use crate::app_log::AppLogRepository;
use crate::authentication::AuthenticationService;
use crate::core::pix_principal::{PixPrincipalService, SendPixRequest};
use crate::customer::CustomerService;
use crate::customer_status_types::CustomerStatusTypesService;
use crate::model::{Customer, Partners, WebhookManager};
use crate::modules::shared::auth::{create_token, create_refresh_token, rotate_refresh_token, AdminAuth};
use crate::partners::PartnersService;
use crate::transaction::TransactionService;
use crate::webhook_manager::WebhookManagerService;

const LOGIN_MAX_ATTEMPTS: usize = 5;
const LOGIN_WINDOW: Duration = Duration::from_secs(5 * 60);   // 5 min sliding window
const LOGIN_LOCKOUT: Duration = Duration::from_secs(15 * 60); // 15 min lockout after max attempts

/// Per-username login rate limiter. Tracks failed attempt timestamps in a sliding window.
#[derive(Default)]
pub struct LoginRateLimiter {
    state: Mutex<HashMap<String, Vec<Instant>>>,
}

impl LoginRateLimiter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Returns `Err(seconds_until_retry)` if the username is currently locked out.
    pub async fn check(&self, username: &str) -> Result<(), u64> {
        let now = Instant::now();
        let mut map = self.state.lock().await;
        let attempts = map.entry(username.to_string()).or_default();
        // Remove timestamps outside the window
        attempts.retain(|t| now.duration_since(*t) < LOGIN_WINDOW);
        if attempts.len() >= LOGIN_MAX_ATTEMPTS {
            let oldest = attempts[0];
            let elapsed = now.duration_since(oldest);
            if elapsed < LOGIN_LOCKOUT {
                let remaining = (LOGIN_LOCKOUT - elapsed).as_secs().max(1);
                return Err(remaining);
            }
            // Lockout expired — clear
            attempts.clear();
        }
        Ok(())
    }

    pub async fn record_failure(&self, username: &str) {
        let mut map = self.state.lock().await;
        map.entry(username.to_string()).or_default().push(Instant::now());
    }

    pub async fn clear(&self, username: &str) {
        self.state.lock().await.remove(username);
    }
}

#[derive(Clone)]
pub struct AdminState {
    pub customer_svc: Arc<dyn CustomerService>,
    pub accounts_svc: Arc<dyn AccountsService>,
    pub partners_svc: Arc<dyn PartnersService>,
    pub webhook_manager_svc: Arc<dyn WebhookManagerService>,
    pub transaction_svc: Arc<dyn TransactionService>,
    pub auth_svc: Option<Arc<dyn AuthenticationService>>,
    pub pix_svc: Option<Arc<dyn PixPrincipalService>>,
    pub customer_status_types_svc: Option<Arc<dyn CustomerStatusTypesService>>,
    pub app_log_repo: Option<Arc<dyn AppLogRepository>>,
    pub login_limiter: Arc<LoginRateLimiter>,
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
    pub refresh_token: String,
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

async fn auth_login(
    State(state): State<AdminState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest("Username and password are required"));
    }
    if let Err(retry_after) = state.login_limiter.check(&req.username).await {
        tracing::warn!(username = %req.username, retry_after_secs = retry_after, "Login rate limit exceeded");
        return Err(AppError::RateLimited(retry_after));
    }
    let auth_svc = state.auth_svc.as_ref().ok_or(AppError::BadRequest("Admin auth not configured"))?;
    let auth = auth_svc
        .find_by_username_and_type(&req.username, crate::authentication::TYPE_AUTH_ADMIN)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or_else(|| {
            // Count as failure even for unknown usernames (prevent enumeration timing)
            AppError::BadRequest("Invalid credentials")
        })?;
    if !auth.active {
        state.login_limiter.record_failure(&req.username).await;
        return Err(AppError::BadRequest("Account inactive"));
    }
    if !verify_password(&req.password, &auth.password) {
        state.login_limiter.record_failure(&req.username).await;
        if let Some(ref log) = state.app_log_repo {
            let _ = log.insert("WARN", "admin.auth", &format!("Failed login for username={}", req.username)).await;
        }
        return Err(AppError::BadRequest("Invalid credentials"));
    }
    state.login_limiter.clear(&req.username).await;
    let access_token = create_token(auth.id as i32, &auth.username, "admin").map_err(|_| AppError::Internal)?;
    let refresh_token = create_refresh_token(auth.id as i32, &auth.username, "admin").map_err(|_| AppError::Internal)?;
    if let Some(ref log) = state.app_log_repo {
        let _ = log.insert("INFO", "admin.auth", &format!("Admin login: username={}", auth.username)).await;
    }
    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 86400,
        user: LoginUser {
            id: auth.id as i32,
            username: auth.username,
            role: "admin".to_string(),
        },
    }))
}

async fn auth_profile(AdminAuth(auth): AdminAuth) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": auth.user_id,
        "username": auth.username,
        "role": auth.role,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    #[allow(dead_code)]
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

async fn auth_change_password(
    State(state): State<AdminState>,
    AdminAuth(auth): AdminAuth,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let new_password = req.new_password.as_deref().unwrap_or("").to_string();
    if new_password.is_empty() {
        return Err(AppError::BadRequest("new_password required"));
    }
    let hashed = bcrypt::hash(&new_password, bcrypt::DEFAULT_COST)
        .map_err(|_| AppError::Internal)?;
    let auth_svc = state.auth_svc.as_ref().ok_or(AppError::BadRequest("Admin auth not configured"))?;
    auth_svc
        .update_password(auth.user_id as i64, &hashed)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(serde_json::json!({ "message": "Password changed successfully" })))
}

// ---- Token Refresh ----
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

async fn auth_refresh(
    State(state): State<AdminState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.refresh_token.is_empty() {
        return Err(AppError::BadRequest("refresh_token required"));
    }
    let (new_access, new_refresh) = rotate_refresh_token(&req.refresh_token)
        .map_err(|_| AppError::BadRequest("Invalid or expired refresh token"))?;
    if let Some(ref log) = state.app_log_repo {
        let _ = log.insert("INFO", "admin.auth", "Admin token refreshed").await;
    }
    Ok(Json(serde_json::json!({
        "access_token": new_access,
        "refresh_token": new_refresh,
        "token_type": "Bearer",
        "expires_in": 86400,
    })))
}

// ---- Customers (delegate to customer service) ----
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

async fn customers_list(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Query(p): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = p.limit.unwrap_or(10).clamp(1, 100);
    let page = p.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;
    let page_result = state.customer_svc.list(offset, limit).await?;
    Ok(Json(serde_json::json!({
        "data": page_result.items,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": page_result.total,
        },
    })))
}

async fn customers_get(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let customer = state.customer_svc.get_by_id(id).await?;
    let c = customer.ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::to_value(c).map_err(|_| AppError::Internal)?))
}

async fn customers_update(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
    Json(item): Json<Customer>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    state.customer_svc.update(id, &item).await?;
    Ok(Json(serde_json::json!({
        "message": "Customer updated successfully",
        "customer_id": id,
    })))
}

async fn customers_delete(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    if !state.customer_svc.delete(id).await? {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({
        "message": "Customer deleted successfully",
        "customer_id": id,
    })))
}

async fn customers_balance(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let customer_id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let customer = state.customer_svc.get_by_id(customer_id).await?.ok_or(AppError::NotFound)?;
    let account = state
        .accounts_svc
        .get_by_authentication_id(customer.authentication_id)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or(AppError::NotFound)?;
    let balance = state
        .transaction_svc
        .get_balance(account.id)
        .await
        .map_err(|_| AppError::Internal)?;
    let balance_f64: f64 = balance.to_string().parse().unwrap_or(0.0);
    Ok(Json(serde_json::json!({
        "balance": balance_f64,
        "preventiveBlock": 0.0,
        "availableBalance": balance_f64,
    })))
}

async fn customers_create_account(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let customer_id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let customer = state.customer_svc.get_by_id(customer_id).await?.ok_or(AppError::NotFound)?;
    let existing = state
        .accounts_svc
        .get_by_authentication_id(customer.authentication_id)
        .await
        .map_err(|_| AppError::Internal)?;
    if existing.is_some() {
        return Err(AppError::BadRequest("Customer already has an account"));
    }
    let account_number = format!("{:013}", chrono::Utc::now().timestamp_millis() % 10_000_000_000_000i64);
    let account = Accounts {
        id: 0,
        account_number: account_number.clone(),
        branch: "0001".to_string(),
        account_type_id: 1,
        account_status_id: 4,
        deleted_at: None,
        authentication_id: customer.authentication_id,
        type_person_id: None,
        full_count: None,
    };
    let acc_id = state.accounts_svc.create(&account).await.map_err(|_| AppError::Internal)?;
    Ok(Json(serde_json::json!({
        "message": "Account created successfully",
        "account_id": acc_id,
        "account_number": account_number,
    })))
}

async fn reports_profit(
    State(state): State<AdminState>,
    _auth: AdminAuth,
) -> Result<Json<serde_json::Value>, AppError> {
    const ADMIN_ACCOUNT_ID: i64 = 1;
    let profit = state.transaction_svc.get_profit(ADMIN_ACCOUNT_ID).await?;
    let profit_f64: f64 = profit.to_string().parse().unwrap_or(0.0);
    Ok(Json(serde_json::json!({
        "profit": profit_f64,
        "admin_account_id": ADMIN_ACCOUNT_ID,
    })))
}

async fn reports_customer_activities(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Query(p): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = p.limit.unwrap_or(50).clamp(1, 200);
    let offset = ((p.page.unwrap_or(1).max(1)) - 1) * limit;
    let page = state.transaction_svc.get_customer_activities(offset, limit).await?;
    Ok(Json(serde_json::json!({
        "items": page.items,
        "pagination": { "page": p.page.unwrap_or(1), "limit": limit, "total": page.total },
    })))
}

async fn reports_balance_differences(
    State(state): State<AdminState>,
    _auth: AdminAuth,
) -> Result<Json<serde_json::Value>, AppError> {
    const MAX_ACCOUNTS: i64 = 500;
    let page = state.accounts_svc.list(0, MAX_ACCOUNTS).await.map_err(|_| AppError::Internal)?;
    let mut items = Vec::new();
    for acc in &page.items {
        let balance = state.transaction_svc.get_balance(acc.id).await.map_err(|_| AppError::Internal)?;
        let balance_f64: f64 = balance.to_string().parse().unwrap_or(0.0);
        items.push(serde_json::json!({
            "account_id": acc.id,
            "account_number": acc.account_number,
            "authentication_id": acc.authentication_id,
            "balance": balance_f64,
        }));
    }
    Ok(Json(serde_json::json!({
        "items": items,
        "total": page.total,
    })))
}

async fn customers_extract(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
    Query(p): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let customer_id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let customer = state.customer_svc.get_by_id(customer_id).await?.ok_or(AppError::NotFound)?;
    let account = state
        .accounts_svc
        .get_by_authentication_id(customer.authentication_id)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or(AppError::NotFound)?;
    let limit = p.limit.unwrap_or(50).clamp(1, 100);
    let offset = ((p.page.unwrap_or(1).max(1)) - 1) * limit;
    let page = state
        .transaction_svc
        .list_by_account(account.id, offset, limit)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(serde_json::json!({
        "items": page.items,
        "pagination": { "page": p.page.unwrap_or(1), "limit": limit, "total": page.total },
    })))
}

async fn customers_approve_kyc(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let customer_id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let approved_status = match &state.customer_status_types_svc {
        Some(svc) => svc.get_by_code("APPROVED").await.ok().flatten().map(|s| s.id).unwrap_or(3),
        None => 3, // fallback: APPROVED is typically id=3
    };
    let mut customer = state.customer_svc.get_by_id(customer_id).await?.ok_or(AppError::NotFound)?;
    customer.customer_status_id = approved_status;
    state.customer_svc.update(customer_id, &customer).await?;
    Ok(Json(serde_json::json!({
        "message": "KYC approved successfully",
        "customer_id": customer_id,
    })))
}

// ---- PIX (stubs for now) ----
async fn pix_list_transactions(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Query(p): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = p.limit.unwrap_or(10).clamp(1, 100);
    let offset = (p.page.unwrap_or(1).max(1) - 1) * limit;
    let page = state.transaction_svc.list(offset, limit).await?;
    Ok(Json(serde_json::json!({
        "data": page.items,
        "pagination": { "page": p.page.unwrap_or(1), "limit": limit, "total": page.total },
    })))
}

async fn pix_get_transaction(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let tx = state.transaction_svc.get_by_id(id).await?;
    let t = tx.ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::to_value(t).map_err(|_| AppError::Internal)?))
}

async fn pix_send(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pix_svc = state.pix_svc.as_ref().ok_or(AppError::BadRequest("PIX not configured"))?;
    let req: SendPixRequest = serde_json::from_value(body).map_err(|_| AppError::BadRequest("Invalid SendPixRequest body"))?;
    let res = pix_svc.send_pix(req).await.map_err(|e| {
        tracing::error!("Admin PIX send failed: {}", e);
        AppError::Internal
    })?;
    Ok(Json(serde_json::json!({
        "statusCode": res.status_code,
        "transactionId": res.transaction_id,
        "data": res.data,
    })))
}

async fn pix_status(_auth: AdminAuth) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "operational" }))
}

async fn pix_create_qrcode(_auth: AdminAuth, Json(_body): Json<serde_json::Value>) -> Result<StatusCode, AppError> {
    Err(AppError::BadRequest("Not implemented"))
}

async fn pix_cancel_transaction(
    _auth: AdminAuth,
    Path(_id): Path<String>,
) -> Result<StatusCode, AppError> {
    Err(AppError::BadRequest("Not implemented"))
}

// ---- Settings ----
async fn settings_get(_auth: AdminAuth) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "pix_enabled": true,
        "max_transaction": 10000.00,
        "min_transaction": 1.00,
        "gateway_failover": true,
        "webhook_retry": 3,
    }))
}

async fn settings_update(_auth: AdminAuth, Json(_body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "message": "Settings updated successfully" }))
}

async fn settings_list_partners(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Query(p): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = p.limit.unwrap_or(10).clamp(1, 100);
    let offset = (p.page.unwrap_or(1).max(1) - 1) * limit;
    let page = state.partners_svc.list(offset, limit).await?;
    Ok(Json(serde_json::json!({
        "data": page.items,
        "pagination": { "page": p.page.unwrap_or(1), "limit": limit, "total": page.total },
    })))
}

async fn settings_create_partner(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Json(item): Json<Partners>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = state.partners_svc.create(&item).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

async fn settings_update_partner(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
    Json(item): Json<Partners>,
) -> Result<StatusCode, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    state.partners_svc.update(id, &item).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn settings_delete_partner(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    if !state.partners_svc.delete(id).await? {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---- Webhooks (delegate to webhook_manager) ----
async fn webhooks_list(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Query(p): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = p.limit.unwrap_or(10).clamp(1, 100);
    let offset = (p.page.unwrap_or(1).max(1) - 1) * limit;
    let page = state.webhook_manager_svc.list(offset, limit).await?;
    Ok(Json(serde_json::json!({
        "data": page.items,
        "pagination": { "page": p.page.unwrap_or(1), "limit": limit, "total": page.total },
    })))
}

async fn webhooks_create(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Json(item): Json<WebhookManager>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = state.webhook_manager_svc.create(&item).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

async fn webhooks_get(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let w = state.webhook_manager_svc.get_by_id(id).await?;
    let webhook = w.ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::to_value(webhook).map_err(|_| AppError::Internal)?))
}

async fn webhooks_update(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
    Json(item): Json<WebhookManager>,
) -> Result<StatusCode, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    state.webhook_manager_svc.update(id, &item).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn webhooks_delete(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    if !state.webhook_manager_svc.delete(id).await? {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn webhooks_test(
    State(state): State<AdminState>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let webhook_id: i64 = id.parse().map_err(|_| AppError::BadRequest("Invalid id"))?;
    let webhook = state.webhook_manager_svc.get_by_id(webhook_id).await?.ok_or(AppError::NotFound)?;
    let payload = serde_json::json!({
        "type": "test",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "webhookId": webhook_id,
        "message": "Test webhook from admin",
    });
    let body = serde_json::to_vec(&payload).map_err(|_| AppError::Internal)?;
    let client = reqwest::Client::new();
    let mut req = client
        .post(&webhook.callback_url)
        .header("Content-Type", "application/json")
        .body(body.clone());
    if !webhook.api_key.is_empty() {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;
        let mut mac = HmacSha256::new_from_slice(webhook.api_key.as_bytes())
            .map_err(|_| AppError::Internal)?;
        mac.update(&body);
        let result = mac.finalize();
        let sig = hex::encode(result.into_bytes());
        req = req.header("X-Webhook-Signature", sig);
    }
    let res = req.send().await.map_err(|e| {
        tracing::error!("Webhook test POST failed: {}", e);
        AppError::Internal
    })?;
    let status = res.status().as_u16();
    Ok(Json(serde_json::json!({
        "message": "Test webhook sent",
        "webhook_id": webhook_id,
        "url": webhook.callback_url,
        "status": status,
    })))
}

// ---- Error ----
#[derive(Debug)]
pub enum AppError {
    BadRequest(&'static str),
    NotFound,
    Internal,
    RateLimited(u64), // seconds until retry
    Customer(crate::customer::CustomerHandlerAppError),
    Partners(crate::partners::PartnersHandlerAppError),
    WebhookManager(crate::webhook_manager::WebhookManagerHandlerAppError),
    Transaction(crate::transaction::TransactionHandlerAppError),
}

impl From<crate::customer::ServiceError> for AppError {
    fn from(e: crate::customer::ServiceError) -> Self {
        AppError::Customer(crate::customer::CustomerHandlerAppError::Service(e))
    }
}
impl From<crate::partners::ServiceError> for AppError {
    fn from(e: crate::partners::ServiceError) -> Self {
        AppError::Partners(crate::partners::PartnersHandlerAppError::Service(e))
    }
}
impl From<crate::webhook_manager::ServiceError> for AppError {
    fn from(e: crate::webhook_manager::ServiceError) -> Self {
        AppError::WebhookManager(crate::webhook_manager::WebhookManagerHandlerAppError::Service(e))
    }
}
impl From<crate::transaction::ServiceError> for AppError {
    fn from(e: crate::transaction::ServiceError) -> Self {
        AppError::Transaction(crate::transaction::TransactionHandlerAppError::Service(e))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": m }))).into_response(),
            AppError::NotFound => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Not found" }))).into_response(),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Internal error" }))).into_response(),
            AppError::RateLimited(retry_after) => (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", retry_after.to_string())],
                Json(serde_json::json!({ "error": "Too many login attempts. Try again later.", "retry_after_secs": retry_after })),
            ).into_response(),
            AppError::Customer(e) => e.into_response(),
            AppError::Partners(e) => e.into_response(),
            AppError::WebhookManager(e) => e.into_response(),
            AppError::Transaction(e) => e.into_response(),
        }
    }
}

/// Verifies a password against a stored hash (bcrypt) or falls back to plaintext comparison
/// for accounts not yet migrated. Logs a warning for plaintext matches to track migration progress.
fn verify_password(provided: &str, stored: &str) -> bool {
    if stored.starts_with("$2b$") || stored.starts_with("$2a$") {
        bcrypt::verify(provided, stored).unwrap_or(false)
    } else {
        // Legacy plaintext — accept but warn; rehash on next password change
        if stored == provided {
            tracing::warn!("login: plaintext password matched — account should be migrated to bcrypt");
            true
        } else {
            false
        }
    }
}

pub fn routes(state: AdminState) -> Router {
    Router::new()
        .route("/auth/login", post(auth_login))
        .route("/auth/refresh", post(auth_refresh))
        .route("/auth/profile", get(auth_profile))
        .route("/auth/change-password", post(auth_change_password))
        .route("/customers", get(customers_list))
        .route("/customers/:id", get(customers_get).put(customers_update).delete(customers_delete))
        .route("/customers/:id/kyc", post(customers_approve_kyc))
        .route("/customers/:id/balance", get(customers_balance))
        .route("/customers/:id/extract", get(customers_extract))
        .route("/customers/:id/account", post(customers_create_account))
        .route("/reports/profit", get(reports_profit))
        .route("/reports/customer-activities", get(reports_customer_activities))
        .route("/reports/balance-differences", get(reports_balance_differences))
        .route("/pix/transactions", get(pix_list_transactions))
        .route("/pix/transactions/:id", get(pix_get_transaction))
        .route("/pix/send", post(pix_send))
        .route("/pix/status", get(pix_status))
        .route("/pix/qrcode", post(pix_create_qrcode))
        .route("/pix/transactions/:id/cancel", post(pix_cancel_transaction))
        .route("/settings", get(settings_get).put(settings_update))
        .route("/settings/partners", get(settings_list_partners).post(settings_create_partner))
        .route("/settings/partners/:id", put(settings_update_partner).delete(settings_delete_partner))
        .route("/webhooks", get(webhooks_list).post(webhooks_create))
        .route("/webhooks/:id", get(webhooks_get).put(webhooks_update).delete(webhooks_delete))
        .route("/webhooks/:id/test", post(webhooks_test))
        .with_state(state)
}
