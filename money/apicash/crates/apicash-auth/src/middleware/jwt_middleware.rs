//! Middleware Axum: gateway público e painel admin.

use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use crate::models::claims::{JwtClaims, Role};
use crate::service::AuthService;

fn extract_bearer(req: &Request) -> Option<&str> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| {
            h.strip_prefix("Bearer ")
                .or_else(|| h.strip_prefix("bearer "))
        })
}

fn extract_api_key_header(req: &Request) -> Option<&str> {
    req.headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
}

/// Rotas do gateway `apicash-core`: JWT HS256 ou modo legado `APICASH_API_KEY` / aberto em dev.
pub async fn verify_core_gateway(
    auth: &AuthService,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if auth.config().auth_disabled {
        return Ok(next.run(req).await);
    }

    if auth.config().jwt_secret.is_empty() {
        return legacy_or_open(auth, req, next).await;
    }

    let Some(token) = extract_bearer(&req) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let claims = auth
        .validate_access_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

async fn legacy_or_open(
    auth: &AuthService,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if auth.config().gateway_legacy_open() {
        return Ok(next.run(req).await);
    }
    let Ok(k) = std::env::var("APICASH_API_KEY") else {
        return Ok(next.run(req).await);
    };
    if k.is_empty() {
        return Ok(next.run(req).await);
    }
    let ok = extract_bearer(&req)
        .or_else(|| extract_api_key_header(&req))
        .is_some_and(|t| t == k.as_str());
    if ok {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Rotas administrativas: `X-API-Key` / Bearer igual a `APICASH_ADMIN_API_KEY`, ou JWT com papel Admin/Platform.
pub async fn verify_admin(
    auth: &AuthService,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if auth.config().auth_disabled {
        return Ok(next.run(req).await);
    }

    if let Ok(expected) = std::env::var("APICASH_ADMIN_API_KEY") {
        if !expected.is_empty() {
            let got = extract_api_key_header(&req).or_else(|| extract_bearer(&req));
            if got == Some(expected.as_str()) {
                return Ok(next.run(req).await);
            }
        }
    }

    if auth.config().jwt_secret.is_empty() {
        tracing::warn!("admin: APICASH_ADMIN_API_KEY missing and no JWT secret — denying");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let Some(token) = extract_bearer(&req) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let claims = auth
        .validate_access_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    if !matches!(claims.role, Role::Admin | Role::Platform) {
        return Err(StatusCode::FORBIDDEN);
    }
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

/// Extrai claims JWT (se existirem) colocadas pelo middleware.
#[must_use]
pub fn claims_from_extensions(ext: &axum::http::Extensions) -> Option<&JwtClaims> {
    ext.get::<JwtClaims>()
}

/// Extracts the authenticated user id from validated JWT claims.
///
/// Security rule: servers must derive the user identity from the JWT (`sub`) inserted by middleware.
#[must_use]
pub fn get_current_user_id(ext: &axum::http::Extensions) -> Option<uuid::Uuid> {
    ext.get::<JwtClaims>().map(|c| c.current_user_id())
}
