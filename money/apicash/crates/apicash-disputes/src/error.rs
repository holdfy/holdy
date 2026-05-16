//! Erros do domínio de disputas.

use apicash_custody::CustodyError;
use apicash_events::EventError;
use apicash_shared::ApiCashError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DisputeError {
    #[error("dispute not found: {0}")]
    NotFound(Uuid),

    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error("custody: {0}")]
    Custody(#[from] CustodyError),

    #[error("messaging: {0}")]
    Event(#[from] EventError),

    #[error("repository: {0}")]
    Repository(String),

    #[error("validation: {0}")]
    Validation(String),
}

impl From<DisputeError> for ApiCashError {
    fn from(e: DisputeError) -> Self {
        match e {
            DisputeError::NotFound(id) => ApiCashError::NotFound(format!("dispute {id}")),
            DisputeError::InvalidState(s) => ApiCashError::Validation(s),
            DisputeError::Custody(c) => c.into(),
            DisputeError::Event(ev) => ApiCashError::Pulsar(ev.to_string()),
            DisputeError::Repository(s) => ApiCashError::Internal(s),
            DisputeError::Validation(s) => ApiCashError::Validation(s),
        }
    }
}
