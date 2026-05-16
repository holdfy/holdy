// JWT auth and role guards for admin, backoffice, customer (from app/modules/shared/middleware/auth.go)
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i32,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: i32,
    username: String,
    role: String,
    #[serde(default)]
    exp: Option<i64>,
    #[serde(default)]
    iat: Option<i64>,
}

/// Builds a JWT string for the given user (for login responses).
pub fn create_token(user_id: i32, username: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = std::time::SystemTime::UNIX_EPOCH
        .elapsed()
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let exp = now + 86400; // 24h
    let claims = Claims {
        user_id,
        username: username.to_string(),
        role: role.to_string(),
        exp: Some(exp),
        iat: Some(now),
    };
    let key = EncodingKey::from_secret(jwt_secret().as_slice());
    encode(&Header::default(), &claims, &key)
}

fn jwt_secret() -> Vec<u8> {
    env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-change-in-production".to_string()).into_bytes()
}

/// Validates Bearer token and returns claims. Caller must check role.
fn validate_token(token_string: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_secret(jwt_secret().as_slice());
    let mut validation = Validation::default();
    validation.validate_exp = true;
    decode::<Claims>(token_string, &key, &validation).map(|d| d.claims)
}

fn bearer_token(parts: &Parts) -> Option<&str> {
    let auth = parts.headers.get(axum::http::header::AUTHORIZATION)?;
    let v = auth.to_str().ok()?;
    v.strip_prefix("Bearer ")
}

/// Extractor that requires a valid JWT with role "admin".
pub struct AdminAuth(pub AuthContext);

#[async_trait]
impl<S> FromRequestParts<S> for AdminAuth
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts).ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Authorization header required" })),
            )
                .into_response()
        })?;
        let claims = validate_token(token).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid token" })),
            )
                .into_response()
        })?;
        if claims.role != "admin" {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "Admin access required" })),
            )
                .into_response());
        }
        Ok(AdminAuth(AuthContext {
            user_id: claims.user_id,
            username: claims.username,
            role: claims.role,
        }))
    }
}

/// Extractor that requires a valid JWT with role "backoffice".
pub struct BackofficeAuth(pub AuthContext);

#[async_trait]
impl<S> FromRequestParts<S> for BackofficeAuth
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts).ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Authorization header required" })),
            )
                .into_response()
        })?;
        let claims = validate_token(token).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid token" })),
            )
                .into_response()
        })?;
        if claims.role != "backoffice" {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "Backoffice access required" })),
            )
                .into_response());
        }
        Ok(BackofficeAuth(AuthContext {
            user_id: claims.user_id,
            username: claims.username,
            role: claims.role,
        }))
    }
}

/// Extractor that requires a valid JWT with role "customer".
pub struct CustomerAuth(pub AuthContext);

#[async_trait]
impl<S> FromRequestParts<S> for CustomerAuth
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts).ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Authorization header required" })),
            )
                .into_response()
        })?;
        let claims = validate_token(token).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid token" })),
            )
                .into_response()
        })?;
        if claims.role != "customer" {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "Customer access required" })),
            )
                .into_response());
        }
        Ok(CustomerAuth(AuthContext {
            user_id: claims.user_id,
            username: claims.username,
            role: claims.role,
        }))
    }
}
