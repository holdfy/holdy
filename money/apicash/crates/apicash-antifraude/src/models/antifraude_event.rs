//! Audit-friendly events emitted by the anti-fraud subsystem.
//! These can be forwarded to Pulsar/Kafka for SIEM integration.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::score::OnRampDecision;

/// Types of persisted audit events (Pulsar/Kafka payloads can mirror this).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AntifraudeEvent {
    ScoreCalculated {
        user_id: Uuid,
        score: u32,
        decision: OnRampDecision,
        at: DateTime<Utc>,
    },
    OnRampBlocked {
        user_id: Uuid,
        reason: String,
        at: DateTime<Utc>,
    },
    /// Fired when a user's transaction count in a time window exceeds the velocity limit.
    VelocityAlert {
        user_id: Uuid,
        tx_count: u32,
        window_hours: u32,
        at: DateTime<Utc>,
    },
    /// Fired when a transaction amount falls in a known structuring band (COAF).
    StructuringAlert {
        user_id: Uuid,
        amount_brl: Decimal,
        at: DateTime<Utc>,
    },
    /// Fired when a counterparty opens a dispute against this user.
    CounterpartyDisputeAlert {
        user_id: Uuid,
        dispute_count: u32,
        at: DateTime<Utc>,
    },
}
