//! Errors returned by the anti-fraud subsystem.

use apicash_shared::ApiCashError;
use thiserror::Error;

/// Failure modes for scoring and validation.
#[derive(Debug, Error)]
pub enum AntiFraudeError {
    #[error("invalid document: {0}")]
    InvalidDocument(String),

    #[error("SEFAZ validation failed: {0}")]
    Sefaz(String),

    #[error("social validation failed: {0}")]
    Social(String),

    #[error("repository error: {0}")]
    Repository(String),

    #[error("HTTP client: {0}")]
    Http(#[from] reqwest::Error),

    #[error("internal: {0}")]
    Internal(String),
}

impl From<AntiFraudeError> for ApiCashError {
    fn from(e: AntiFraudeError) -> Self {
        match e {
            AntiFraudeError::InvalidDocument(m) => ApiCashError::Validation(m),
            AntiFraudeError::Sefaz(m) => ApiCashError::AntiFraud(m),
            AntiFraudeError::Social(m) => ApiCashError::AntiFraud(m),
            AntiFraudeError::Repository(m) => ApiCashError::Internal(m),
            AntiFraudeError::Http(err) => ApiCashError::External(err.to_string()),
            AntiFraudeError::Internal(m) => ApiCashError::Internal(m),
        }
    }
}
