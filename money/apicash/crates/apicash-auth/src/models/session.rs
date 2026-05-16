//! Metadados de sessão / token.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Informação de sessão após login (metadados opcionais; tokens vêm em [`crate::service::LoginResponse`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
