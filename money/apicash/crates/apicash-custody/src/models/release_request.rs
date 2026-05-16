//! Release workflow payloads (off-chain coordinator today; Soroban `invoke` later).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Confirmation required before releasing escrow (idempotent release in production).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfirmation {
    pub released_by: Uuid,
    pub idempotency_key: String,
}

/// Outcome after a successful release: yield split recorded for settlement rails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseResult {
    pub custody_id: Uuid,
    pub order_id: Uuid,
    pub yield_distributed: super::YieldDistribution,
}
