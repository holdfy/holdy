//! User score snapshot and coarse risk bucket.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::risk_factors::RiskFactor;

/// Coarse risk bucket used for routing and Stellar on-ramp gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    /// Highest risk tier — typically blocks automated on-ramp.
    Critical,
}

/// Outcome for Stellar funding / anchor flows after scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnRampDecision {
    Approve,
    Review,
    Block,
}

/// Aggregated anti-fraud score for a user (0–1000 inclusive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScore {
    pub user_id: Uuid,
    /// Raw score on the 0–1000 scale.
    pub score: u32,
    pub risk_level: RiskLevel,
    pub factors: Vec<RiskFactor>,
    pub last_updated: DateTime<Utc>,
    /// Recommended action before allowing on-ramp to Stellar.
    pub decision: OnRampDecision,
}

impl UserScore {
    /// Recomendação estável para políticas e integrações externas (`APPROVE` / `REVIEW` / `BLOCK`).
    #[must_use]
    pub fn get_risk_recommendation(&self) -> &'static str {
        match self.decision {
            OnRampDecision::Approve => "APPROVE",
            OnRampDecision::Review => "REVIEW",
            OnRampDecision::Block => "BLOCK",
        }
    }
}
