//! Minimal user context for fraud checks.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lightweight profile slice used when scoring (expand with KYC fields later).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: Uuid,
    /// Open disputes tied to this user (penalised in scoring).
    pub open_dispute_count: u32,
}
