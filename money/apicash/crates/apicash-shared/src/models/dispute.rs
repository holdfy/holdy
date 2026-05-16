//! Dispute case linked to an order.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::DisputeStatus;

/// A formal dispute opened against an order in custody or after release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dispute {
    pub id: Uuid,
    pub order_id: Uuid,
    pub opened_by: Uuid,
    pub status: DisputeStatus,
    pub reason_code: String,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}
