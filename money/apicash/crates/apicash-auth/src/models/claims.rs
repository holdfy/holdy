//! Claims JWT e papéis.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Papel do utilizador na plataforma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Seller,
    Buyer,
    Admin,
    Platform,
}

impl Role {
    #[must_use]
    pub fn is_admin_or_platform(self) -> bool {
        matches!(self, Role::Admin | Role::Platform)
    }
}

/// Claims serializadas no JWT (access ou refresh).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Utilizador (`sub` padrão JWT).
    pub sub: Uuid,
    /// Convenience duplicate of `sub` for app-level payloads.
    ///
    /// Security note: do not trust this over `sub`; servers should always derive the current user
    /// from the validated JWT `sub`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    pub role: Role,
    /// Score agregado 0–1000 (antifraude), se conhecido no momento da emissão.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_score: Option<u32>,
    /// `Some("refresh")` para refresh tokens; omitido ou `access` para tokens de API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_use: Option<String>,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
}

impl JwtClaims {
    /// Returns the authenticated user id (derived from JWT `sub`).
    #[must_use]
    pub fn current_user_id(&self) -> Uuid {
        self.sub
    }

    #[must_use]
    pub fn is_refresh_token(&self) -> bool {
        self.token_use.as_deref() == Some("refresh")
    }

    /// Regras de negócio: operações sensíveis exigem score mínimo ou papel elevado.
    #[must_use]
    pub fn can_operate_high_risk(&self) -> bool {
        self.role.is_admin_or_platform() || self.risk_score.is_some_and(|s| s >= 400)
    }
}
