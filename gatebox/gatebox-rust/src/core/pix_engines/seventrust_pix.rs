// SevenTrust PIX engine - wraps SevenTrustHttpService for PIX flows
use std::sync::Arc;

use crate::core::gateways::services::{GatewayHttpService, SevenTrustHttpService};

/// SevenTrust PIX engine: uses SevenTrustHttpService (stub) for PIX flows.
pub type SevenTrustPixEngine = Arc<dyn GatewayHttpService>;

/// Build SevenTrust PIX engine.
#[allow(dead_code)]
pub fn new_seventrust_pix_engine() -> SevenTrustPixEngine {
    Arc::new(SevenTrustHttpService::default())
}
