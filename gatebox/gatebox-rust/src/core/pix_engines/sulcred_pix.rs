// Sulcred PIX engine - wraps SulcredHttpService for PIX flows
use std::sync::Arc;

use crate::core::gateways::services::{GatewayHttpService, SulcredHttpService};

/// Sulcred PIX engine: uses SulcredHttpService for send_pix_key, get_balance, get_token_out.
pub type SulcredPixEngine = Arc<dyn GatewayHttpService>;

/// Build Sulcred PIX engine (Sulcred HTTP service as GatewayHttpService).
#[allow(dead_code)]
pub fn new_sulcred_pix_engine() -> SulcredPixEngine {
    Arc::new(SulcredHttpService::default())
}
