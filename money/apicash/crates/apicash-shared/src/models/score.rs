//! User risk score and factor breakdown.

use std::fmt;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Coarse risk bucket for routing and limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Blocked,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Medium => write!(f, "medium"),
            RiskLevel::High => write!(f, "high"),
            RiskLevel::Blocked => write!(f, "blocked"),
        }
    }
}

/// Named factor contributing to the aggregate score (weights are not necessarily 0–1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreFactor {
    pub name: String,
    pub weight: Decimal,
}

/// Anti-fraud / behavior score snapshot for a user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserScore {
    pub user_id: Uuid,
    /// Aggregate score (domain-specific scale; compare to [`crate::constants::USER_SCORE_MAX`]).
    pub score: u32,
    pub risk_level: RiskLevel,
    pub last_updated: DateTime<Utc>,
    pub factors: Vec<ScoreFactor>,
}
