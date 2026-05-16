//! Liberação de custódia via WhatsApp — desativada até implementação futura.

use crate::core_api::{CoreApiClient, CoreApiError};
use crate::session::UserSession;

/// Custódia não é libertada pelo canal WhatsApp nesta fase.
pub async fn try_confirm_and_release(
    _core: &CoreApiClient,
    _session: &mut UserSession,
    _text: &str,
    _bearer: &str,
) -> Result<Option<Vec<String>>, CoreApiError> {
    Ok(None)
}
