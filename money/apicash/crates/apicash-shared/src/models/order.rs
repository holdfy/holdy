//! Order aggregate (escrow trade).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{Money, OrderStatus};

/// A bilateral order between buyer and seller, funded into custody.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    /// Principal amount held for this order (same currency as custody).
    pub amount: Money,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    /// Security/business rule helper: only the **buyer** can confirm delivery and authorize escrow release.
    #[must_use]
    pub fn is_buyer(&self, user_id: &Uuid) -> bool {
        &self.buyer_id == user_id
    }

    /// Helper: check if the given user is the seller for this order.
    #[must_use]
    pub fn is_seller(&self, user_id: &Uuid) -> bool {
        &self.seller_id == user_id
    }
}
