//! Enumerated risk factors with explicit point deltas (weights).

use serde::{Deserialize, Serialize};

/// Typed factors applied to the aggregate score (positive = trust, negative = risk).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RiskFactor {
    /// CPF/CNPJ regular with Receita Federal / SEFAZ-style clearance.
    SefazStatus {
        /// Points added when status is regular (e.g. +300).
        weight: i32,
    },
    /// Social profile age signal (months since creation, platform-specific).
    SocialAccountAge {
        platform: String,
        months: u32,
        weight: i32,
    },
    /// Historical on-platform transaction behaviour (placeholder for PSP data).
    TransactionHistory { weight: i32 },
    /// Open or recent disputes strongly penalise on-ramp eligibility.
    DisputeHistory { open_disputes: u32, weight: i32 },
    /// Generic adjustment (limits, velocity, device fingerprint, etc.).
    Other { code: String, weight: i32 },
}
