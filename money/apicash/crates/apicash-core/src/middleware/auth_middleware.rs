//! Gateway: delega para `apicash-auth` (JWT ou legado `APICASH_API_KEY`).

use std::sync::Arc;

use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

use apicash_auth::middleware::verify_core_gateway;

use crate::state::AppState;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    verify_core_gateway(&state.auth, req, next).await
}
