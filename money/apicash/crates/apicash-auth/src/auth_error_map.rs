//! Conversão para [`apicash_shared::ApiCashError`] em limites de serviço.

use apicash_shared::ApiCashError;

use crate::service::AuthError;

impl From<AuthError> for ApiCashError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::InvalidCredentials => ApiCashError::Auth("invalid credentials".into()),
            AuthError::InvalidToken(m) => ApiCashError::Auth(m),
            AuthError::Misconfigured => ApiCashError::Auth("JWT misconfigured".into()),
            AuthError::Internal(m) => ApiCashError::Internal(m),
        }
    }
}
