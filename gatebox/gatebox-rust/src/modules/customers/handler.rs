// From app/modules/customers/handler - auth, account, pix, p2p (pix + auth + account wired)
use axum::{extract::{Path, Query, State}, routing::get, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::accounts::AccountsService;
use crate::authentication::AuthenticationService;
use crate::core::pix_principal::{
    GenerateQrCodeRequest, PixPrincipalService, PixWebhookService, SendPixRequest, SendReversalRequest,
};
use crate::modules::shared::auth::create_token;
use crate::modules::shared::CustomerAuth;

use crate::p2p::P2PService;
use crate::webhook_manager::WebhookManagerService;

#[derive(Clone)]
pub struct CustomersState {
    pub pix_svc: Arc<dyn PixPrincipalService>,
    pub webhook_svc: Option<Arc<dyn PixWebhookService>>,
    pub auth_svc: Option<Arc<dyn AuthenticationService>>,
    pub accounts_svc: Option<Arc<dyn AccountsService>>,
    pub p2p_svc: Option<Arc<dyn P2PService>>,
    pub webhook_manager_svc: Option<Arc<dyn WebhookManagerService>>,
}

pub fn routes(state: CustomersState) -> Router {
    Router::new()
        .nest("/auth", auth_routes())
        .nest("/account", account_routes())
        .nest("/pix", pix_routes())
        .nest("/p2p", p2p_routes())
        .with_state(state)
}

fn auth_routes() -> Router<CustomersState> {
    Router::new()
        .route("/login", post(auth_login))
        .route("/register", post(auth_register))
        .route("/profile", get(auth_profile).put(auth_update_profile))
        .route("/change-password", post(auth_change_password))
}

fn account_routes() -> Router<CustomersState> {
    Router::new()
        .route("/balance", get(account_balance))
        .route("/extract", get(account_extract))
        .route("/limits", get(account_limits))
        .route("/keys", get(account_list_keys).post(account_create_key))
        .route("/keys/:id", axum::routing::delete(account_delete_key))
        .route("/webhooks", get(account_webhooks_list).post(account_webhooks_create))
}

fn pix_routes() -> Router<CustomersState> {
    Router::new()
        .route("/send", post(pix_send))
        .route("/decode-brcode", post(pix_decode_brcode))
        .route("/qrcode", post(pix_qrcode))
        .route("/status", get(pix_status))
        .route("/transactions", get(pix_transactions))
        .route("/reversal", post(pix_reversal))
}

fn p2p_routes() -> Router<CustomersState> {
    Router::new()
        .route("/send", post(p2p_send))
        .route("/history", get(p2p_history))
        .route("/status/:transfer_id", get(p2p_status))
        .route("/search", get(p2p_search))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginBody {
    username: Option<String>,
    password: Option<String>,
}

async fn auth_login(
    State(state): State<CustomersState>,
    Json(body): Json<LoginBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let username = body.username.as_deref().unwrap_or("").to_string();
    let password = body.password.as_deref().unwrap_or("").to_string();
    if username.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "username required".to_string()));
    }
    let Some(auth_svc) = &state.auth_svc else {
        return Ok(Json(json!({ "token": "", "message": "auth not configured" })));
    };
    let auth = auth_svc
        .find_by_username(&username)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::UNAUTHORIZED, "invalid credentials".to_string()))?;
    if !auth.active {
        return Err((axum::http::StatusCode::UNAUTHORIZED, "account inactive".to_string()));
    }
    if auth.password != password {
        return Err((axum::http::StatusCode::UNAUTHORIZED, "invalid credentials".to_string()));
    }
    let token = create_token(auth.id as i32, &auth.username, "customer")
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({
        "accessToken": token,
        "tokenType": "Bearer",
    })))
}
async fn auth_register() -> Json<serde_json::Value> {
    Json(json!({ "message": "stub" }))
}
async fn auth_profile(_auth: CustomerAuth) -> Json<serde_json::Value> {
    Json(json!({ "userId": _auth.0.user_id, "username": _auth.0.username }))
}
async fn auth_update_profile(_auth: CustomerAuth) -> Json<serde_json::Value> {
    Json(json!({ "message": "stub" }))
}
async fn auth_change_password(_auth: CustomerAuth) -> Json<serde_json::Value> {
    Json(json!({ "message": "stub" }))
}

async fn account_balance(
    State(state): State<CustomersState>,
    auth: CustomerAuth,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let auth_id = auth.0.user_id as i64;
    let balance = if let Some(accounts_svc) = &state.accounts_svc {
        let _account = accounts_svc
            .get_by_authentication_id(auth_id)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        // Model Accounts has no balance field; use 0 or future balance service
        0.0
    } else {
        0.0
    };
    Ok(Json(json!({ "balance": balance })))
}
async fn account_extract(_auth: CustomerAuth) -> Json<serde_json::Value> {
    Json(json!({ "items": [] }))
}
async fn account_limits(_auth: CustomerAuth) -> Json<serde_json::Value> {
    Json(json!({ "limits": {} }))
}
async fn account_list_keys(_auth: CustomerAuth) -> Json<serde_json::Value> {
    Json(json!({ "keys": [] }))
}
async fn account_create_key(_auth: CustomerAuth) -> Json<serde_json::Value> {
    Json(json!({ "message": "stub" }))
}
async fn account_delete_key(_auth: CustomerAuth, _: axum::extract::Path<String>) -> Json<serde_json::Value> {
    Json(json!({ "message": "stub" }))
}

async fn account_webhooks_list(
    State(state): State<CustomersState>,
    auth: CustomerAuth,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let accounts_svc = state.accounts_svc.as_ref().ok_or_else(|| (axum::http::StatusCode::NOT_IMPLEMENTED, "accounts not configured".to_string()))?;
    let webhook_svc = state.webhook_manager_svc.as_ref().ok_or_else(|| (axum::http::StatusCode::NOT_IMPLEMENTED, "webhooks not configured".to_string()))?;
    let account = accounts_svc
        .get_by_authentication_id(auth.0.user_id as i64)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "account not found".to_string()))?;
    let items = webhook_svc
        .list_by_account(account.id, 0, 50)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let safe_items: Vec<serde_json::Value> = items
        .into_iter()
        .map(|w| json!({
            "id": w.id,
            "callbackUrl": w.callback_url,
            "webhookTypeId": w.webhook_type_id,
            "accountId": w.account_id,
        }))
        .collect();
    Ok(Json(json!({ "items": safe_items })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebhookCreateBody {
    callback_url: Option<String>,
    webhook_type_id: Option<i64>,
}

async fn account_webhooks_create(
    State(state): State<CustomersState>,
    auth: CustomerAuth,
    Json(body): Json<WebhookCreateBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let accounts_svc = state.accounts_svc.as_ref().ok_or_else(|| (axum::http::StatusCode::NOT_IMPLEMENTED, "accounts not configured".to_string()))?;
    let webhook_svc = state.webhook_manager_svc.as_ref().ok_or_else(|| (axum::http::StatusCode::NOT_IMPLEMENTED, "webhooks not configured".to_string()))?;
    let account = accounts_svc
        .get_by_authentication_id(auth.0.user_id as i64)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "account not found".to_string()))?;
    let callback_url = body.callback_url.as_deref().unwrap_or("").trim().to_string();
    let webhook_type_id = body.webhook_type_id.unwrap_or(1);
    if callback_url.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "callbackUrl required".to_string()));
    }
    if !callback_url.starts_with("http://") && !callback_url.starts_with("https://") {
        return Err((axum::http::StatusCode::BAD_REQUEST, "callbackUrl must start with http:// or https://".to_string()));
    }
    let item = crate::model::WebhookManager {
        id: 0,
        callback_url: callback_url.clone(),
        username: String::new(),
        password: String::new(),
        api_key: String::new(),
        webhook_type_id,
        account_id: account.id,
        deleted_at: None,
        full_count: None,
    };
    let id = webhook_svc
        .create(&item)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({
        "id": id,
        "callbackUrl": callback_url,
        "webhookTypeId": webhook_type_id,
        "accountId": account.id,
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PixSendBody {
    account: Option<String>,
    bank: Option<String>,
    document_number: Option<String>,
    amount: Option<f64>,
    branch: Option<String>,
    key: Option<String>,
    name: Option<String>,
    external_id: Option<String>,
    memo: Option<String>,
    type_key: Option<String>,
}

async fn pix_send(
    State(s): State<CustomersState>,
    auth: CustomerAuth,
    Json(body): Json<PixSendBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let req = SendPixRequest {
        account: body.account.unwrap_or_default(),
        bank: body.bank.unwrap_or_default(),
        document_number: body.document_number.unwrap_or_default(),
        amount: body.amount.unwrap_or(0.0),
        branch: body.branch.unwrap_or_default(),
        key: body.key.unwrap_or_default(),
        name: body.name.unwrap_or_default(),
        external_id: body.external_id,
        memo: body.memo,
        type_key: body.type_key,
        user_id: Some(auth.0.user_id as i64),
    };
    let res = s.pix_svc.send_pix(req).await.map_err(|e| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(serde_json::json!({
        "statusCode": res.status_code,
        "transactionId": res.transaction_id,
        "data": res.data,
    })))
}
async fn pix_decode_brcode(_auth: CustomerAuth) -> Json<serde_json::Value> {
    Json(json!({ "message": "stub" }))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PixQrcodeBody {
    amount: Option<f64>,
    payer_name: Option<String>,
    payer_document: Option<String>,
    description: Option<String>,
    expiration_seconds: Option<i32>,
    reference: Option<String>,
    pix_key: Option<String>,
}
async fn pix_qrcode(
    State(s): State<CustomersState>,
    _auth: CustomerAuth,
    Json(body): Json<PixQrcodeBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let req = GenerateQrCodeRequest {
        amount: body.amount.unwrap_or(0.0),
        payer_name: body.payer_name.unwrap_or_default(),
        payer_document: body.payer_document.unwrap_or_default(),
        description: body.description.unwrap_or_default(),
        expiration_seconds: body.expiration_seconds.unwrap_or(1800),
        reference: body.reference.unwrap_or_default(),
        pix_key: body.pix_key,
    };
    let res = s.pix_svc.generate_qr_code(req).await.map_err(|e| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(serde_json::json!({
        "statusCode": res.status_code,
        "qrCode": res.qr_code,
        "txId": res.tx_id,
        "expiresAt": res.expires_at,
        "transactionId": res.transaction_id,
        "gateway": res.gateway,
        "data": res.data,
    })))
}
async fn pix_status(_auth: CustomerAuth) -> Json<serde_json::Value> {
    Json(json!({ "status": "stub" }))
}
async fn pix_transactions(_auth: CustomerAuth) -> Json<serde_json::Value> {
    Json(json!({ "items": [] }))
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReversalBody {
    end2end: String,
    amount: f64,
    external_id: Option<String>,
}

async fn pix_reversal(
    State(state): State<CustomersState>,
    auth: CustomerAuth,
    Json(body): Json<ReversalBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let svc = state
        .webhook_svc
        .as_ref()
        .ok_or_else(|| (axum::http::StatusCode::NOT_IMPLEMENTED, "reversal not configured".to_string()))?;
    let req = SendReversalRequest {
        end2end: body.end2end,
        amount: body.amount,
        external_id: body.external_id.unwrap_or_default(),
    };
    let res = svc
        .send_reversal(auth.0.user_id as i64, req)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            let status = if msg.contains("not found") || msg.contains("insufficient") || msg.contains("error in amount") {
                axum::http::StatusCode::BAD_REQUEST
            } else {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, msg)
        })?;
    Ok(Json(serde_json::json!({
        "statusCode": res.status_code,
        "transactionId": res.transaction_id,
        "data": res.data,
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct P2PSendBody {
    receiver_id: Option<i64>,
    amount: Option<f64>,
    description: Option<String>,
}

async fn p2p_send(
    State(state): State<CustomersState>,
    auth: CustomerAuth,
    Json(body): Json<P2PSendBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let Some(p2p_svc) = &state.p2p_svc else {
        return Err((axum::http::StatusCode::NOT_IMPLEMENTED, "P2P not configured".to_string()));
    };
    let receiver_id = body.receiver_id.ok_or_else(|| (axum::http::StatusCode::BAD_REQUEST, "receiverId required".to_string()))?;
    let amount = rust_decimal::Decimal::try_from(body.amount.unwrap_or(0.0))
        .map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "invalid amount".to_string()))?;
    let res = p2p_svc
        .p2p_send(auth.0.user_id as i64, receiver_id, amount, body.description)
        .await
        .map_err(|e| {
            let status = match &e {
                crate::p2p::P2PError::InsufficientBalance | crate::p2p::P2PError::InvalidAmount => axum::http::StatusCode::BAD_REQUEST,
                crate::p2p::P2PError::ReceiverNotFound | crate::p2p::P2PError::ReceiverAccountNotFound | crate::p2p::P2PError::SenderAccountNotFound => axum::http::StatusCode::NOT_FOUND,
                _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, e.to_string())
        })?;
    Ok(Json(serde_json::json!({
        "transferId": res.transfer_id,
        "status": res.status,
    })))
}

#[derive(Debug, Deserialize)]
struct P2PHistoryQuery {
    page: Option<i64>,
    limit: Option<i64>,
}

async fn p2p_history(
    State(state): State<CustomersState>,
    auth: CustomerAuth,
    Query(q): Query<P2PHistoryQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let Some(p2p_svc) = &state.p2p_svc else {
        return Err((axum::http::StatusCode::NOT_IMPLEMENTED, "P2P not configured".to_string()));
    };
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let offset = (q.page.unwrap_or(1).max(1) - 1) * limit;
    let items = p2p_svc
        .p2p_history(auth.0.user_id as i64, offset, limit)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({
        "items": items,
    })))
}

async fn p2p_status(
    State(state): State<CustomersState>,
    auth: CustomerAuth,
    Path(transfer_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let Some(p2p_svc) = &state.p2p_svc else {
        return Err((axum::http::StatusCode::NOT_IMPLEMENTED, "P2P not configured".to_string()));
    };
    let transfer_id: i64 = transfer_id.parse().map_err(|_| (axum::http::StatusCode::BAD_REQUEST, "invalid transfer_id".to_string()))?;
    let tx = p2p_svc
        .p2p_status(transfer_id, auth.0.user_id as i64)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match tx {
        Some(t) => Ok(Json(serde_json::json!({
            "id": t.id,
            "status": if t.status_transaction_id == 4 { "completed" } else { "pending" },
            "amount": t.amount,
            "accountId": t.account_id,
        }))),
        None => Err((axum::http::StatusCode::NOT_FOUND, "transfer not found".to_string())),
    }
}

#[derive(Debug, Deserialize)]
struct P2PSearchQuery {
    q: Option<String>,
}

async fn p2p_search(
    State(state): State<CustomersState>,
    auth: CustomerAuth,
    Query(q): Query<P2PSearchQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let _ = auth;
    let Some(p2p_svc) = &state.p2p_svc else {
        return Err((axum::http::StatusCode::NOT_IMPLEMENTED, "P2P not configured".to_string()));
    };
    let query = q.q.as_deref().unwrap_or("");
    let results = p2p_svc
        .p2p_search(query)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({
        "results": results,
    })))
}
