//! Custody / escrow bucket for locked funds.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{CustodyStatus, Money};

/// Funds locked for an order, including optional yield accrual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Custody {
    pub order_id: Uuid,
    pub amount: Money,
    pub status: CustodyStatus,
    pub locked_at: DateTime<Utc>,
    pub released_at: Option<DateTime<Utc>>,
    /// Yield credited while in custody (policy-specific).
    pub yield_earned: Money,
}
