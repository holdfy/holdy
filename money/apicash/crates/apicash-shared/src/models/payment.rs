//! Payment records (fiat or on-chain legs).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{Money, PaymentStatus};

/// A single payment attempt or capture linked to an order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub order_id: Uuid,
    pub amount: Money,
    pub status: PaymentStatus,
    /// External PSP or chain reference when available.
    pub external_reference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
