//! Login, refresh token, registro e emissão de JWT.

use std::str::FromStr;
use std::sync::Arc;

use apicash_auth::service::{LoginRequest, LoginResponse, RefreshRequest};
use apicash_auth::{AuthError, PersonType, Role};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use tracing::instrument;

use crate::error::ApiError;
use crate::state::AppState;

/// Mantém só os dígitos de um CPF/CNPJ.
fn normalize_document(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Validação matemática de CPF/CNPJ (mesmo algoritmo do front-end, `site/src/lib/format.ts`).
fn is_valid_document(digits: &str) -> bool {
    fn check_digit(digits: &[u8], len: usize) -> u32 {
        let sum: u32 = digits[..len]
            .iter()
            .enumerate()
            .map(|(i, &d)| d as u32 * (len + 1 - i) as u32)
            .sum();
        let r = 11 - (sum % 11);
        if r >= 10 { 0 } else { r }
    }
    fn check_digit_cnpj(digits: &[u8], len: usize) -> u32 {
        let weights_12 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
        let weights_13 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
        let weights: &[u32] = if len == 12 { &weights_12 } else { &weights_13 };
        let sum: u32 = digits[..len]
            .iter()
            .zip(weights)
            .map(|(&d, &w)| d as u32 * w)
            .sum();
        let r = 11 - (sum % 11);
        if r >= 10 { 0 } else { r }
    }

    let bytes: Vec<u8> = digits.chars().filter_map(|c| c.to_digit(10)).map(|d| d as u8).collect();
    if bytes.len() != digits.len() {
        return false;
    }
    match digits.len() {
        11 => {
            if bytes.iter().all(|&d| d == bytes[0]) {
                return false;
            }
            check_digit(&bytes, 9) == bytes[9] as u32 && check_digit(&bytes, 10) == bytes[10] as u32
        }
        14 => {
            if bytes.iter().all(|&d| d == bytes[0]) {
                return false;
            }
            check_digit_cnpj(&bytes, 12) == bytes[12] as u32
                && check_digit_cnpj(&bytes, 13) == bytes[13] as u32
        }
        _ => false,
    }
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

fn role_str(role: Role) -> String {
    format!("{role:?}").to_lowercase()
}

#[instrument(skip(state, req))]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    match state.auth.login_with_risk_score(&req.username, &req.password).await {
        Ok(resp) => Ok(Json(resp)),
        // Não está na lista estática `APICASH_AUTH_USERS` — tenta usuário
        // cadastrado via /auth/register (CPF/CNPJ + senha, persistido em `users`).
        Err(AuthError::InvalidCredentials) => {
            login_with_document(&state, &req.username, &req.password)
                .await
                .map(Json)
        }
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn login_with_document(
    state: &AppState,
    username: &str,
    password: &str,
) -> Result<LoginResponse, ApiError> {
    let document = normalize_document(username);
    let invalid = || ApiError::unauthorized("credenciais inválidas");
    if document.is_empty() {
        return Err(invalid());
    }
    let user = state
        .user_repository
        .find_by_document(&document)
        .await
        .ok_or_else(invalid)?;
    let hash = user.password_hash.as_deref().ok_or_else(invalid)?;
    if !verify_password(password, hash) {
        return Err(invalid());
    }

    let role = Role::from_str(&user.role).unwrap_or(Role::Buyer);
    let person_type = PersonType::from_document(&document);
    let access = state
        .auth
        .generate_token_full(user.id, role, person_type, document.clone(), None)
        .map_err(ApiError::from)?;
    let refresh = state
        .auth
        .generate_refresh_token(user.id, role, person_type, document)
        .map_err(ApiError::from)?;
    Ok(LoginResponse::new(
        access,
        state.auth.config().jwt_ttl_secs,
        refresh,
        state.auth.config().jwt_refresh_ttl_secs,
    ))
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

/// `POST /auth/register` — cadastro self-service (CPF/CNPJ + senha).
/// Persiste em `users` (tabela também usada pelo login social) com senha
/// hasheada (Argon2) e devolve o mesmo par access+refresh do `/auth/login`.
#[instrument(skip(state, req))]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let document = normalize_document(&req.document);
    if !is_valid_document(&document) {
        return Err(ApiError::bad_request("documento inválido — verifique os dígitos"));
    }
    if req.password.trim().len() < 4 {
        return Err(ApiError::bad_request("senha muito curta"));
    }
    if state.user_repository.find_by_document(&document).await.is_some() {
        return Err(ApiError::conflict("documento já cadastrado"));
    }

    let role = match req.role.as_deref().unwrap_or("buyer") {
        "seller" => Role::Seller,
        "admin" => Role::Admin,
        _ => Role::Buyer,
    };
    let password_hash = hash_password(&req.password).map_err(ApiError::internal)?;

    let user = state
        .user_repository
        .create_with_password(&document, &password_hash, &role_str(role), req.name.as_deref())
        .await
        .map_err(|e| {
            if e.contains("cadastrado") {
                ApiError::conflict(e)
            } else {
                ApiError::internal(e)
            }
        })?;

    tracing::info!(user_id = %user.id, role = %role_str(role), "register: novo usuário (documento+senha)");

    let person_type = PersonType::from_document(&document);
    let access = state
        .auth
        .generate_token_full(user.id, role, person_type, document.clone(), None)
        .map_err(ApiError::from)?;
    let refresh = state
        .auth
        .generate_refresh_token(user.id, role, person_type, document)
        .map_err(ApiError::from)?;

    Ok(Json(LoginResponse::new(
        access,
        state.auth.config().jwt_ttl_secs,
        refresh,
        state.auth.config().jwt_refresh_ttl_secs,
    )))
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub document: String,
    pub password: String,
    pub name: Option<String>,
    pub role: Option<String>,
}
