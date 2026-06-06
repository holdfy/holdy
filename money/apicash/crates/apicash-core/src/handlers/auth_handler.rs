//! Login, refresh token, registro e emissão de JWT.

use std::sync::Arc;

use apicash_auth::service::{LoginRequest, LoginResponse, RefreshRequest};
use apicash_auth::Role;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

#[instrument(skip(state, req))]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let resp = state
        .auth
        .login_with_risk_score(&req.username, &req.password)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(resp))
}

#[instrument(skip(state, req))]
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let resp = state
        .auth
        .refresh_tokens(&req.refresh_token)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(resp))
}

/// `POST /auth/register` — cria identidade nova e devolve JWT.
/// MVP: sem persistência de senha; usa UUID determinístico do e-mail.
/// Para produção: adicionar tabela `users` com bcrypt + validação de unicidade.
#[instrument(skip(state, req))]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    let role: Role = match req.role.as_deref().unwrap_or("seller") {
        "buyer"  => Role::Buyer,
        "admin"  => Role::Admin,
        _        => Role::Seller,
    };
    let user_id = Uuid::new_v5(
        &Uuid::NAMESPACE_DNS,
        format!("holdy:user:{}", req.email.to_lowercase().trim()).as_bytes(),
    );
    let access_token = state
        .auth
        .generate_token(user_id, role, None)
        .map_err(ApiError::from)?;

    tracing::info!(
        user_id = %user_id,
        email   = %req.email,
        role    = ?role,
        "register: novo usuário"
    );

    Ok(Json(RegisterResponse {
        access_token,
        token_type: "Bearer",
        user_id: user_id.to_string(),
        role: format!("{role:?}").to_lowercase(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub user_id: String,
    pub role: String,
}
