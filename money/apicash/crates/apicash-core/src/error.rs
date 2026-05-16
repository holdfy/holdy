//! HTTP API errors.

use apicash_shared::ApiCashError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: msg.into(),
        }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: msg.into(),
        }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.into(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.into(),
        }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: msg.into(),
        }
    }

    /// Upstream anchor / settlement dependency missing or invalid (HTTP 502).
    pub fn bad_gateway(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: msg.into(),
        }
    }
}

impl From<ApiCashError> for ApiError {
    fn from(e: ApiCashError) -> Self {
        use apicash_shared::ApiCashError::*;
        match e {
            Validation(m) | Money(m) => ApiError::bad_request(m),
            NotFound(m) => ApiError::not_found(m),
            Conflict(m) => ApiError::conflict(m),
            Auth(m) => ApiError::unauthorized(m),
            UnauthorizedRelease => ApiError::forbidden(e.to_string()),
            Config(_) | Serialization(_) | Uuid(_) | Io(_) | External(_) | Stellar(_)
            | Pulsar(_) | WhatsApp(_) | AntiFraud(_) | Internal(_) => {
                ApiError::internal(e.to_string())
            }
        }
    }
}

impl From<apicash_custody::CustodyError> for ApiError {
    fn from(e: apicash_custody::CustodyError) -> Self {
        ApiError::from(ApiCashError::from(e))
    }
}

impl From<apicash_anchor::AnchorError> for ApiError {
    fn from(e: apicash_anchor::AnchorError) -> Self {
        ApiError::from(ApiCashError::from(e))
    }
}

impl From<apicash_auth::AuthError> for ApiError {
    fn from(e: apicash_auth::AuthError) -> Self {
        ApiError::from(ApiCashError::from(e))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({ "error": self.message });
        (self.status, Json(body)).into_response()
    }
}
