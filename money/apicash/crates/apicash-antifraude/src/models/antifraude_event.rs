//! Audit-friendly events emitted by the anti-fraud subsystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::score::OnRampDecision;

/// Types of persisted audit events (Kafka/Pulsar payloads can mirror this).
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
}
