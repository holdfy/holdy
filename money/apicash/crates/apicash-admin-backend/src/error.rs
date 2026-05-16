//! Erros HTTP do painel administrativo.

use apicash_shared::ApiCashError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("unauthorized")]
    Unauthorized,

    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("{0}")]
    ApiCash(#[from] ApiCashError),
}

impl From<apicash_custody::CustodyError> for AdminError {
    fn from(e: apicash_custody::CustodyError) -> Self {
        AdminError::ApiCash(ApiCashError::from(e))
    }
}

impl From<apicash_disputes::error::DisputeError> for AdminError {
    fn from(e: apicash_disputes::error::DisputeError) -> Self {
        AdminError::ApiCash(ApiCashError::from(e))
    }
}

impl AdminError {
    pub fn internal(msg: impl Into<String>) -> Self {
        AdminError::ApiCash(ApiCashError::Internal(msg.into()))
    }
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        let status = match &self {
            AdminError::Unauthorized => StatusCode::UNAUTHORIZED,
            AdminError::NotFound(_) => StatusCode::NOT_FOUND,
            AdminError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AdminError::ApiCash(e) => status_for_apicash(e),
        };
        let body = ErrorBody {
            error: self.to_string(),
        };
        (status, Json(body)).into_response()
    }
}

fn status_for_apicash(e: &ApiCashError) -> StatusCode {
    use apicash_shared::ApiCashError::*;
    match e {
        Validation(_) | Money(_) => StatusCode::BAD_REQUEST,
        NotFound(_) => StatusCode::NOT_FOUND,
        Conflict(_) => StatusCode::CONFLICT,
        Auth(_) => StatusCode::UNAUTHORIZED,
        UnauthorizedRelease => StatusCode::FORBIDDEN,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
