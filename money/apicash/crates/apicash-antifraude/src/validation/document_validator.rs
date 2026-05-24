//! Core trait and types for interchangeable document validation (CPF / CNPJ).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AntiFraudeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentType {
    Cpf,
    Cnpj,
}

impl DocumentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cpf => "cpf",
            Self::Cnpj => "cnpj",
        }
    }
}

/// Result of validating a CPF or CNPJ document.
///
/// `Valid` / `Invalid` are definitive and cacheable.
/// `Unknown` means the provider could not determine status — never block on this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Valid,
    Invalid,
    Unknown,
}

/// Pluggable document validation backend.
///
/// Implementations: [`LocalDocumentValidator`], [`HttpDocumentValidator`],
/// [`CachedDocumentValidator`].
#[async_trait]
pub trait DocumentValidator: Send + Sync {
    async fn validate(
        &self,
        document: &str,
        doc_type: DocumentType,
    ) -> Result<DocumentStatus, AntiFraudeError>;
}
