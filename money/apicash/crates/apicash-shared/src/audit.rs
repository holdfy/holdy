//! Structured audit events for sensitive actions.
//!
//! These events are designed to be logged (via `tracing`) at service boundaries so we can answer:
//! "who did what, to which order, and what was the outcome?"

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// High-signal audit events across services.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum AuditEvent {
    OrderCreated {
        user_id: Uuid,
        order_id: Uuid,
        buyer_id: Uuid,
        seller_id: Uuid,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    FundsLocked {
        user_id: Uuid,
        order_id: Uuid,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    DeliveryConfirmed {
        user_id: Uuid,
        order_id: Uuid,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    FundsReleased {
        user_id: Uuid,
        order_id: Uuid,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    DisputeOpened {
        user_id: Uuid,
        order_id: Uuid,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    UnauthorizedAttempt {
        user_id: Option<Uuid>,
        order_id: Option<Uuid>,
        action: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
}
