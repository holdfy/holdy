//! Login, refresh token e emissão de JWT (score antifraude quando disponível).

use std::sync::Arc;

use apicash_auth::service::{LoginRequest, LoginResponse, RefreshRequest};
use axum::extract::State;
use axum::Json;
use tracing::instrument;

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
