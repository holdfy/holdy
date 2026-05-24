//! Enumerated risk factors with explicit point deltas (weights).

use serde::{Deserialize, Serialize};

/// Typed factors applied to the aggregate score (positive = trust, negative = risk).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RiskFactor {
    /// CPF/CNPJ status with Receita Federal / SEFAZ-style clearance.
    SefazStatus { weight: i32 },

    /// Social profile age signal (months since creation, platform-specific).
    SocialAccountAge {
        platform: String,
        months: u32,
        weight: i32,
    },

    /// Dispute opened BY this user (chargeback / active litigation).
    DisputeHistory { open_disputes: u32, weight: i32 },

    /// Dispute(s) opened AGAINST this user by a counterparty.
    CounterpartyDisputes { count: u32, weight: i32 },

    /// Ratio of disputes to total transactions exceeds the safe threshold.
    DisputeRate { rate_pct: u32, weight: i32 },

    /// Number of orders in a given time window exceeds velocity limits.
    VelocityCheck {
        tx_count: u32,
        window_hours: u32,
        weight: i32,
    },

    /// BRL volume transacted in a time window triggers a high-volume flag.
    HighVolume {
        amount_brl: String,
        window_hours: u32,
        weight: i32,
    },

    /// Transaction amount falls in a structuring band (COAF threshold avoidance).
    Structuring { amount_brl: String, weight: i32 },

    /// Account maturity: trust bonus for established users, penalty for new accounts.
    AccountMaturity {
        tx_count: u32,
        age_days: u32,
        weight: i32,
    },

    /// Current transaction amount is anomalously high relative to historical average.
    ValueAnomaly { ratio_pct: u32, weight: i32 },

    /// CNPJ registration status at Receita Federal (active vs. inactive/suspended).
    CnpjStatus { active: bool, weight: i32 },

    /// Company age in months — trust bonus for established companies, penalty for new ones.
    CompanyAge { months: u32, weight: i32 },

    /// Generic adjustment (IP, device fingerprint, etc.).
    Other { code: String, weight: i32 },
}
