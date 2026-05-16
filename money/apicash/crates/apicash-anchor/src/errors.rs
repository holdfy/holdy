//! Errors from Anchor / Horizon / Soroban integration.

use apicash_shared::ApiCashError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnchorError {
    #[error("configuration: {0}")]
    Config(String),

    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("horizon: {0}")]
    Horizon(String),

    #[error("anchor: {0}")]
    Anchor(String),

    #[error("invalid input: {0}")]
    Validation(String),

    #[error("provider unavailable: {0}")]
    Unavailable(String),
}

impl From<AnchorError> for ApiCashError {
    fn from(e: AnchorError) -> Self {
        match e {
            AnchorError::Config(m) => ApiCashError::External(format!("anchor config: {m}")),
            AnchorError::Http(err) => ApiCashError::External(err.to_string()),
            AnchorError::Json(err) => ApiCashError::Serialization(err),
            AnchorError::Horizon(m) => ApiCashError::Stellar(m),
            AnchorError::Anchor(m) => ApiCashError::Stellar(m),
            AnchorError::Validation(m) => ApiCashError::Validation(m),
            AnchorError::Unavailable(m) => ApiCashError::External(m),
        }
    }
}
