//! Serviço de autenticação: login, emissão e validação de JWT.

use std::str::FromStr;

use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "antifraude")]
use std::sync::Arc;

#[cfg(feature = "antifraude")]
use apicash_antifraude::AntiFraudeService;

use crate::config::AuthConfig;
use crate::models::claims::{JwtClaims, Role};
use crate::models::user::UserIdentity;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("credenciais inválidas")]
    InvalidCredentials,
    #[error("token inválido ou expirado: {0}")]
    InvalidToken(String),
    #[error("configuração JWT incompleta")]
    Misconfigured,
    #[error("erro interno: {0}")]
    Internal(String),
}

/// Serviço central de autenticação.
#[derive(Clone)]
pub struct AuthService {
    config: AuthConfig,
    #[cfg(feature = "antifraude")]
    antifraude: Option<Arc<AntiFraudeService>>,
}

impl AuthService {
    #[must_use]
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "antifraude")]
            antifraude: None,
        }
    }

    #[cfg(feature = "antifraude")]
    #[must_use]
    pub fn with_antifraude(config: AuthConfig, antifraude: Arc<AntiFraudeService>) -> Self {
        Self {
            config,
            antifraude: Some(antifraude),
        }
    }

    #[must_use]
    pub fn config(&self) -> &AuthConfig {
        &self.config
    }

    /// Valida utilizador/senha (lista `APICASH_AUTH_USERS`: `user:pass:role`, separado por `;`).
    pub fn login(&self, username: &str, password: &str) -> Result<LoginResponse, AuthError> {
        let user = self.resolve_user(username, password)?;
        let access = self.generate_token(user.id, user.role, None)?;
        let refresh = self.generate_refresh_token(user.id, user.role)?;
        Ok(LoginResponse::new(
            access,
            self.config.jwt_ttl_secs,
            refresh,
            self.config.jwt_refresh_ttl_secs,
        ))
    }

    /// Login com score de risco obtido do antifraude (lista de scores em memória).
    #[cfg(feature = "antifraude")]
    pub async fn login_with_risk_score(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LoginResponse, AuthError> {
        let user = self.resolve_user(username, password)?;
        let risk = if let Some(ref af) = self.antifraude {
            let scores = af
                .list_scores()
                .await
                .map_err(|e| AuthError::Internal(e.to_string()))?;
            scores
                .iter()
                .find(|s| s.user_id == user.id)
                .map(|s| s.score)
        } else {
            None
        };
        let access = self.generate_token(user.id, user.role, risk)?;
        let refresh = self.generate_refresh_token(user.id, user.role)?;
        Ok(LoginResponse::new(
            access,
            self.config.jwt_ttl_secs,
            refresh,
            self.config.jwt_refresh_ttl_secs,
        ))
    }

    /// Troca refresh por novo par access + refresh (rotação simples).
    #[cfg(feature = "antifraude")]
    pub async fn refresh_with_risk_score(
        &self,
        refresh_token: &str,
    ) -> Result<LoginResponse, AuthError> {
        let claims = self.validate_token(refresh_token)?;
        if !claims.is_refresh_token() {
            return Err(AuthError::InvalidToken("expected refresh token".into()));
        }
        let risk = if let Some(ref af) = self.antifraude {
            let scores = af
                .list_scores()
                .await
                .map_err(|e| AuthError::Internal(e.to_string()))?;
            scores
                .iter()
                .find(|s| s.user_id == claims.sub)
                .map(|s| s.score)
        } else {
            None
        };
        let access = self.generate_token(claims.sub, claims.role, risk)?;
        let refresh = self.generate_refresh_token(claims.sub, claims.role)?;
        Ok(LoginResponse::new(
            access,
            self.config.jwt_ttl_secs,
            refresh,
            self.config.jwt_refresh_ttl_secs,
        ))
    }

    #[cfg(not(feature = "antifraude"))]
    pub fn refresh_without_antifraude(
        &self,
        refresh_token: &str,
    ) -> Result<LoginResponse, AuthError> {
        let claims = self.validate_token(refresh_token)?;
        if !claims.is_refresh_token() {
            return Err(AuthError::InvalidToken("expected refresh token".into()));
        }
        let access = self.generate_token(claims.sub, claims.role, None)?;
        let refresh = self.generate_refresh_token(claims.sub, claims.role)?;
        Ok(LoginResponse::new(
            access,
            self.config.jwt_ttl_secs,
            refresh,
            self.config.jwt_refresh_ttl_secs,
        ))
    }

    /// Renova tokens (antifraude opcional).
    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<LoginResponse, AuthError> {
        #[cfg(feature = "antifraude")]
        {
            self.refresh_with_risk_score(refresh_token).await
        }
        #[cfg(not(feature = "antifraude"))]
        {
            self.refresh_without_antifraude(refresh_token)
        }
    }

    /// Emite JWT HS256 (access).
    pub fn generate_token(
        &self,
        user_id: Uuid,
        role: Role,
        risk_score: Option<u32>,
    ) -> Result<String, AuthError> {
        if self.config.jwt_secret.len() < 8 {
            return Err(AuthError::Misconfigured);
        }
        let now = Utc::now().timestamp();
        let exp = now + self.config.jwt_ttl_secs as i64;
        let claims = JwtClaims {
            sub: user_id,
            user_id: Some(user_id),
            role,
            risk_score,
            token_use: None,
            exp,
            iat: now,
            iss: self.config.jwt_issuer.clone(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::InvalidToken(e.to_string()))
    }

    /// Refresh token (não usar em rotas protegidas como Bearer de API).
    pub fn generate_refresh_token(&self, user_id: Uuid, role: Role) -> Result<String, AuthError> {
        if self.config.jwt_secret.len() < 8 {
            return Err(AuthError::Misconfigured);
        }
        let now = Utc::now().timestamp();
        let exp = now + self.config.jwt_refresh_ttl_secs as i64;
        let claims = JwtClaims {
            sub: user_id,
            user_id: Some(user_id),
            role,
            risk_score: None,
            token_use: Some("refresh".into()),
            exp,
            iat: now,
            iss: self.config.jwt_issuer.clone(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::InvalidToken(e.to_string()))
    }

    /// Valida e devolve claims (access ou refresh).
    pub fn validate_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        if self.config.jwt_secret.len() < 8 {
            return Err(AuthError::Misconfigured);
        }
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.config.jwt_issuer]);
        let data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        Ok(data.claims)
    }

    /// Valida apenas **access** token (rejeita refresh em middleware de API).
    pub fn validate_access_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let c = self.validate_token(token)?;
        if c.is_refresh_token() {
            return Err(AuthError::InvalidToken(
                "refresh token cannot be used as API bearer".into(),
            ));
        }
        Ok(c)
    }

    fn resolve_user(&self, username: &str, password: &str) -> Result<UserIdentity, AuthError> {
        let spec = std::env::var("APICASH_AUTH_USERS")
            .unwrap_or_else(|_| "admin:admin:admin;seller:seller:seller;buyer:buyer:buyer".into());
        for entry in spec.split(';') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let mut p = entry.split(':');
            let u = p.next().unwrap_or("").trim();
            let pw = p.next().unwrap_or("").trim();
            let role_s = p.next().unwrap_or("buyer").trim();
            if u != username || pw != password {
                continue;
            }
            let role = Role::from_str(role_s).map_err(|_| AuthError::InvalidCredentials)?;
            let id = Uuid::new_v5(
                &Uuid::NAMESPACE_DNS,
                format!("apicash:user:{username}").as_bytes(),
            );
            return Ok(UserIdentity {
                id,
                username: username.to_string(),
                role,
            });
        }
        Err(AuthError::InvalidCredentials)
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "seller" => Ok(Role::Seller),
            "buyer" => Ok(Role::Buyer),
            "admin" => Ok(Role::Admin),
            "platform" => Ok(Role::Platform),
            _ => Err(()),
        }
    }
}

/// Payload JSON de login (API).
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub refresh_token: String,
    pub refresh_expires_in: u64,
}

impl LoginResponse {
    #[must_use]
    pub fn new(
        access_token: String,
        expires_in: u64,
        refresh_token: String,
        refresh_expires_in: u64,
    ) -> Self {
        Self {
            access_token,
            token_type: "Bearer",
            expires_in,
            refresh_token,
            refresh_expires_in,
        }
    }
}

/// Corpo `POST /auth/refresh`.
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
