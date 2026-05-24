//! Identidade de utilizador (login).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::claims::{PersonType, Role};

/// Utilizador autenticável (referência interna).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    pub id: Uuid,
    pub username: String,
    pub role: Role,
    /// Tipo de pessoa: física (CPF) ou jurídica (CNPJ).
    pub person_type: PersonType,
    /// Documento do utilizador (CPF ou CNPJ, dígitos apenas).
    pub document: String,
}
