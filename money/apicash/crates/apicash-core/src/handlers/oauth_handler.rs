//! Login social via OAuth 2.0 / OIDC.
//!
//! Fluxo:
//!   GET /auth/oauth/:provider          → redireciona ao provedor com state CSRF
//!   GET /auth/oauth/:provider/callback → troca code por token, upsert usuário, emite JWT HoldFy,
//!                                        redireciona ao frontend com ?token=&refresh=

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Extension;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use jsonwebtoken::{encode as jwt_encode, Algorithm, EncodingKey, Header as JwtHeader};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{info, warn};
use url::Url;
use uuid::Uuid;

use apicash_auth::{JwtClaims, models::claims::{PersonType, Role}};

use crate::error::ApiError;
use crate::repository::user_repository::{OAuthUser, UserRepository};
use crate::state::AppState;

// ── State CSRF em memória (TTL 10 min, limpeza lazy) ─────────────────────────

#[derive(Debug, Clone)]
pub struct OAuthStateEntry {
    pub provider: String,
    pub created_at: u64,
}

pub type OAuthStates = Arc<Mutex<HashMap<String, OAuthStateEntry>>>;

pub fn new_oauth_states() -> OAuthStates {
    Arc::new(Mutex::new(HashMap::new()))
}

async fn store_state(states: &OAuthStates, state: String, provider: &str) {
    let now = unix_now();
    let mut map = states.lock().await;
    map.retain(|_, v| now - v.created_at < 600);
    map.insert(state, OAuthStateEntry { provider: provider.to_string(), created_at: now });
}

async fn consume_state(states: &OAuthStates, state: &str, provider: &str) -> bool {
    let mut map = states.lock().await;
    match map.remove(state) {
        Some(entry) if entry.provider == provider => unix_now() - entry.created_at < 600,
        _ => false,
    }
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

// ── Redirect ao provedor ──────────────────────────────────────────────────────

pub async fn oauth_redirect(
    Path(provider): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let cfg = &state.oauth;

    let auth_url = match provider.as_str() {
        "google" => {
            let Some(client_id) = &cfg.google_client_id else {
                return ApiError::bad_request("Google OAuth não configurado (GOOGLE_CLIENT_ID ausente)").into_response();
            };
            let csrf = Uuid::new_v4().to_string();
            store_state(&state.oauth_states, csrf.clone(), "google").await;
            let mut u = Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
            u.query_pairs_mut()
                .append_pair("client_id", client_id)
                .append_pair("redirect_uri", &cfg.redirect_uri("google"))
                .append_pair("response_type", "code")
                .append_pair("scope", "openid email profile")
                .append_pair("state", &csrf)
                .append_pair("access_type", "offline")
                .append_pair("prompt", "consent");
            u.to_string()
        }
        "facebook" => {
            let Some(client_id) = &cfg.facebook_client_id else {
                return ApiError::bad_request("Facebook OAuth não configurado (FACEBOOK_CLIENT_ID ausente)").into_response();
            };
            let csrf = Uuid::new_v4().to_string();
            store_state(&state.oauth_states, csrf.clone(), "facebook").await;
            let mut u = Url::parse("https://www.facebook.com/v20.0/dialog/oauth").unwrap();
            u.query_pairs_mut()
                .append_pair("client_id", client_id)
                .append_pair("redirect_uri", &cfg.redirect_uri("facebook"))
                .append_pair("state", &csrf)
                .append_pair("scope", "email,public_profile");
            u.to_string()
        }
        "linkedin" => {
            let Some(client_id) = &cfg.linkedin_client_id else {
                return ApiError::bad_request("LinkedIn OAuth não configurado (LINKEDIN_CLIENT_ID ausente)").into_response();
            };
            let csrf = Uuid::new_v4().to_string();
            store_state(&state.oauth_states, csrf.clone(), "linkedin").await;
            let mut u = Url::parse("https://www.linkedin.com/oauth/v2/authorization").unwrap();
            u.query_pairs_mut()
                .append_pair("client_id", client_id)
                .append_pair("redirect_uri", &cfg.redirect_uri("linkedin"))
                .append_pair("response_type", "code")
                .append_pair("state", &csrf)
                .append_pair("scope", "openid profile email");
            u.to_string()
        }
        "apple" => {
            let Some(client_id) = &cfg.apple_client_id else {
                return ApiError::bad_request("Apple OAuth não configurado (APPLE_CLIENT_ID ausente)").into_response();
            };
            let csrf = Uuid::new_v4().to_string();
            store_state(&state.oauth_states, csrf.clone(), "apple").await;
            let mut u = Url::parse("https://appleid.apple.com/auth/authorize").unwrap();
            u.query_pairs_mut()
                .append_pair("client_id", client_id)
                .append_pair("redirect_uri", &cfg.redirect_uri("apple"))
                .append_pair("response_type", "code")
                .append_pair("state", &csrf)
                .append_pair("scope", "name email")
                .append_pair("response_mode", "form_post");
            u.to_string()
        }
        _ => {
            return ApiError::bad_request(format!("Provedor OAuth desconhecido: {provider}")).into_response();
        }
    };

    Redirect::temporary(&auth_url).into_response()
}

// ── Callback do provedor ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

pub async fn oauth_callback(
    Path(provider): Path<String>,
    Query(query): Query<CallbackQuery>,
    State(app): State<Arc<AppState>>,
) -> Response {
    let frontend = app.oauth.frontend_url.clone();

    if let Some(err) = &query.error {
        warn!(provider = %provider, error = %err, "OAuth callback com erro do provedor");
        return redirect_error(&frontend, &format!("oauth_error:{err}"));
    }

    let (Some(code), Some(state_param)) = (query.code.as_deref(), query.state.as_deref()) else {
        return redirect_error(&frontend, "oauth_missing_params");
    };

    if !consume_state(&app.oauth_states, state_param, &provider).await {
        warn!(provider = %provider, "OAuth state CSRF inválido ou expirado");
        return redirect_error(&frontend, "oauth_state_invalid");
    }

    let result = match provider.as_str() {
        "google"   => handle_google(code, &app).await,
        "facebook" => handle_facebook(code, &app).await,
        "linkedin" => handle_linkedin(code, &app).await,
        "apple"    => handle_apple(code, &app).await,
        _          => Err("provedor_desconhecido".to_string()),
    };

    match result {
        Ok((access, refresh)) => {
            let url = format!("{frontend}/login?token={access}&refresh={refresh}&oauth=1");
            Redirect::temporary(&url).into_response()
        }
        Err(e) => {
            warn!(provider = %provider, error = %e, "OAuth callback falhou");
            redirect_error(&frontend, &format!("oauth_failed:{e}"))
        }
    }
}

fn redirect_error(frontend: &str, reason: &str) -> Response {
    Redirect::temporary(&format!("{frontend}/login?oauth_error={reason}")).into_response()
}

// ── Vincular documento (pós-login social) ────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LinkDocumentRequest {
    pub document: String,
}

#[derive(Debug, Serialize)]
pub struct LinkDocumentResponse {
    pub ok: bool,
}

pub async fn link_document(
    State(app): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    Json(req): Json<LinkDocumentRequest>,
) -> Result<Json<LinkDocumentResponse>, ApiError> {
    let digits: String = req.document.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 11 && digits.len() != 14 {
        return Err(ApiError::bad_request(
            "Documento inválido: informe CPF (11 dígitos) ou CNPJ (14 dígitos)",
        ));
    }
    app.user_repository
        .update_document(claims.sub, &digits)
        .await
        .map_err(ApiError::internal)?;
    info!(user_id = %claims.sub, doc_len = digits.len(), "documento vinculado ao usuário social");
    Ok(Json(LinkDocumentResponse { ok: true }))
}

// ── Handlers por provedor ─────────────────────────────────────────────────────

async fn handle_google(code: &str, app: &Arc<AppState>) -> Result<(String, String), String> {
    let cfg = &app.oauth;
    let client_id     = cfg.google_client_id.as_deref().ok_or("GOOGLE_CLIENT_ID ausente")?;
    let client_secret = cfg.google_client_secret.as_deref().ok_or("GOOGLE_CLIENT_SECRET ausente")?;

    let token: GoogleTokenResponse = app.http
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code",          code),
            ("client_id",     client_id),
            ("client_secret", client_secret),
            ("redirect_uri",  cfg.redirect_uri("google").as_str()),
            ("grant_type",    "authorization_code"),
        ])
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    let userinfo: GoogleUserInfo = app.http
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(&token.access_token)
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    let user = find_or_create_user(
        &*app.user_repository, "google", &userinfo.sub,
        userinfo.email, userinfo.name, userinfo.picture,
    ).await?;
    emit_jwt(app, user)
}

async fn handle_facebook(code: &str, app: &Arc<AppState>) -> Result<(String, String), String> {
    let cfg = &app.oauth;
    let client_id     = cfg.facebook_client_id.as_deref().ok_or("FACEBOOK_CLIENT_ID ausente")?;
    let client_secret = cfg.facebook_client_secret.as_deref().ok_or("FACEBOOK_CLIENT_SECRET ausente")?;

    let token: FacebookTokenResponse = app.http
        .get("https://graph.facebook.com/v20.0/oauth/access_token")
        .query(&[
            ("client_id",     client_id),
            ("client_secret", client_secret),
            ("redirect_uri",  cfg.redirect_uri("facebook").as_str()),
            ("code",          code),
        ])
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    let userinfo: FacebookUserInfo = app.http
        .get("https://graph.facebook.com/me")
        .query(&[("fields", "id,name,email,picture.width(200)")])
        .bearer_auth(&token.access_token)
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    let pic = userinfo.picture.and_then(|p| p.data).map(|d| d.url);
    let user = find_or_create_user(
        &*app.user_repository, "facebook", &userinfo.id,
        userinfo.email, userinfo.name, pic,
    ).await?;
    emit_jwt(app, user)
}

async fn handle_linkedin(code: &str, app: &Arc<AppState>) -> Result<(String, String), String> {
    let cfg = &app.oauth;
    let client_id     = cfg.linkedin_client_id.as_deref().ok_or("LINKEDIN_CLIENT_ID ausente")?;
    let client_secret = cfg.linkedin_client_secret.as_deref().ok_or("LINKEDIN_CLIENT_SECRET ausente")?;

    let token: LinkedInTokenResponse = app.http
        .post("https://www.linkedin.com/oauth/v2/accessToken")
        .form(&[
            ("grant_type",    "authorization_code"),
            ("code",          code),
            ("client_id",     client_id),
            ("client_secret", client_secret),
            ("redirect_uri",  cfg.redirect_uri("linkedin").as_str()),
        ])
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    let userinfo: LinkedInUserInfo = app.http
        .get("https://api.linkedin.com/v2/userinfo")
        .bearer_auth(&token.access_token)
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    let user = find_or_create_user(
        &*app.user_repository, "linkedin", &userinfo.sub,
        userinfo.email, userinfo.name, userinfo.picture,
    ).await?;
    emit_jwt(app, user)
}

async fn handle_apple(code: &str, app: &Arc<AppState>) -> Result<(String, String), String> {
    let cfg = &app.oauth;
    let client_id = cfg.apple_client_id.as_deref().ok_or("APPLE_CLIENT_ID ausente")?;
    let team_id   = cfg.apple_team_id.as_deref().ok_or("APPLE_TEAM_ID ausente")?;
    let key_id    = cfg.apple_key_id.as_deref().ok_or("APPLE_KEY_ID ausente")?;
    let pk_p8     = cfg.apple_private_key_p8.as_deref().ok_or("APPLE_PRIVATE_KEY_P8 ausente")?;

    let client_secret = build_apple_client_secret(team_id, client_id, key_id, pk_p8)?;

    #[derive(Deserialize)]
    struct AppleTokenResp { id_token: String }

    let token: AppleTokenResp = app.http
        .post("https://appleid.apple.com/auth/token")
        .form(&[
            ("client_id",     client_id),
            ("client_secret", client_secret.as_str()),
            ("redirect_uri",  cfg.redirect_uri("apple").as_str()),
            ("code",          code),
            ("grant_type",    "authorization_code"),
        ])
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    // Decodifica id_token sem verificar assinatura (MVP; em prod verificar JWKS da Apple)
    let payload = decode_jwt_payload_insecure(&token.id_token)?;
    let sub: String = payload["sub"].as_str().ok_or("Apple: sub ausente no id_token")?.to_string();
    let email = payload["email"].as_str().map(|s| s.to_string());

    let user = find_or_create_user(
        &*app.user_repository, "apple", &sub,
        email, None, None,
    ).await?;
    emit_jwt(app, user)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn find_or_create_user(
    repo: &dyn UserRepository,
    provider: &str,
    provider_id: &str,
    email: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
) -> Result<OAuthUser, String> {
    if let Some(user) = repo.find_by_provider(provider, provider_id).await {
        return Ok(user);
    }
    if let Some(ref e) = email {
        if let Some(existing) = repo.find_by_email(e).await {
            repo.link_provider(existing.id, provider, provider_id).await?;
            return Ok(existing);
        }
    }
    let new_user = OAuthUser {
        id: Uuid::new_v4(),
        email,
        name,
        avatar_url,
        document: None,
        role: "buyer".into(),
        password_hash: None,
    };
    let created = repo.upsert(new_user).await?;
    repo.link_provider(created.id, provider, provider_id).await?;
    info!(user_id = %created.id, provider, "novo usuário social criado");
    Ok(created)
}

fn emit_jwt(app: &Arc<AppState>, user: OAuthUser) -> Result<(String, String), String> {
    let role = match user.role.as_str() {
        "seller"   => Role::Seller,
        "admin"    => Role::Admin,
        "platform" => Role::Platform,
        _          => Role::Buyer,
    };
    let person_type = user.document.as_deref()
        .map(PersonType::from_document)
        .unwrap_or_default();
    let document = user.document.clone().unwrap_or_default();

    let access = app.auth
        .generate_token_full_with_profile(
            user.id, role, person_type, document.clone(), None,
            user.email.clone(), user.name.clone(), user.avatar_url.clone(),
        )
        .map_err(|e| e.to_string())?;

    let refresh = app.auth
        .generate_refresh_token_with_profile(
            user.id, role, person_type, document,
            user.email, user.name, user.avatar_url,
        )
        .map_err(|e| e.to_string())?;

    Ok((access, refresh))
}

/// Decodifica o payload de um JWT sem verificar a assinatura (base64url → JSON).
fn decode_jwt_payload_insecure(token: &str) -> Result<serde_json::Value, String> {
    use base64::Engine;
    let parts: Vec<&str> = token.split('.').collect();
    let b64 = parts.get(1).ok_or("JWT inválido: faltam partes")?;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(b64)
        .or_else(|_| {
            let padded = format!("{b64}{}", "=".repeat((4 - b64.len() % 4) % 4));
            base64::engine::general_purpose::URL_SAFE.decode(&padded)
        })
        .map_err(|e| format!("base64 decode: {e}"))?;
    serde_json::from_slice(&decoded).map_err(|e| format!("JSON parse: {e}"))
}

/// Constrói o client_secret JWT para Apple Sign In (ES256, chave EC P-256 `.p8`).
fn build_apple_client_secret(team_id: &str, client_id: &str, key_id: &str, pk_pem: &str) -> Result<String, String> {
    let now = unix_now() as i64;

    #[derive(Serialize)]
    struct AppleJwtClaims {
        iss: String,
        iat: i64,
        exp: i64,
        aud: String,
        sub: String,
    }

    let claims = AppleJwtClaims {
        iss: team_id.to_string(),
        iat: now,
        exp: now + 15_777_000, // 6 meses
        aud: "https://appleid.apple.com".to_string(),
        sub: client_id.to_string(),
    };

    let mut header = JwtHeader::new(Algorithm::ES256);
    header.kid = Some(key_id.to_string());

    let key = EncodingKey::from_ec_pem(pk_pem.as_bytes())
        .map_err(|e| format!("Apple p8 EC key parse: {e}"))?;

    jwt_encode(&header, &claims, &key).map_err(|e| format!("Apple JWT sign: {e}"))
}

// ── DTOs dos provedores ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GoogleTokenResponse { access_token: String }

#[derive(Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    picture: Option<String>,
}

#[derive(Deserialize)]
struct FacebookTokenResponse { access_token: String }

#[derive(Deserialize)]
struct FacebookUserInfo {
    id: String,
    name: Option<String>,
    email: Option<String>,
    picture: Option<FacebookPicture>,
}

#[derive(Deserialize)]
struct FacebookPicture { data: Option<FacebookPictureData> }

#[derive(Deserialize)]
struct FacebookPictureData { url: String }

#[derive(Deserialize)]
struct LinkedInTokenResponse { access_token: String }

#[derive(Deserialize)]
struct LinkedInUserInfo {
    sub: String,
    name: Option<String>,
    email: Option<String>,
    picture: Option<String>,
}
