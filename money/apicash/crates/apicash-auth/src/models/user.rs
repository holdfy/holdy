//! Identidade de utilizador (login).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::claims::Role;

/// Utilizador autenticável (referência interna).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    pub id: Uuid,
    pub username: String,
    pub role: Role,
}
