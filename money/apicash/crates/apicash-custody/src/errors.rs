//! Custody domain errors.

use apicash_shared::ApiCashError;
use thiserror::Error;

/// Errors from custody operations (persistence, yield math, release policy).
#[derive(Debug, Error)]
pub enum CustodyError {
    #[error("custody not found for order: {0}")]
    NotFound(uuid::Uuid),

    #[error("invalid state transition: {0}")]
    InvalidState(String),

    #[error("yield calculation: {0}")]
    YieldCalculation(String),

    #[error("arithmetic overflow or invalid money")]
    Arithmetic,

    #[error("repository: {0}")]
    Repository(String),

    #[error("validation: {0}")]
    Validation(String),

    /// Business rule violation: only the buyer can authorize escrow release.
    #[error("unauthorized release (only buyer may release)")]
    UnauthorizedRelease,

    #[error("soroban: {0}")]
    Soroban(String),
}

impl From<CustodyError> for ApiCashError {
    fn from(e: CustodyError) -> Self {
        match e {
            CustodyError::NotFound(id) => ApiCashError::NotFound(format!("custody for order {id}")),
            CustodyError::InvalidState(s) => ApiCashError::Validation(s),
            CustodyError::YieldCalculation(s) => ApiCashError::Validation(s),
            CustodyError::Arithmetic => ApiCashError::Money("custody arithmetic overflow".into()),
            CustodyError::Repository(s) => ApiCashError::Internal(s),
            CustodyError::Validation(s) => ApiCashError::Validation(s),
            CustodyError::UnauthorizedRelease => ApiCashError::UnauthorizedRelease,
            CustodyError::Soroban(s) => ApiCashError::Stellar(s),
        }
    }
}
