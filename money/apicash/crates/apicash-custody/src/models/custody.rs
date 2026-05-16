//! Custody aggregate — funds locked until release policy is satisfied.
//!
//! Future: balances and state transitions will mirror **Soroban** escrow contracts on Stellar.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use apicash_shared::Money;

/// Lifecycle of a custody record (escrow bucket).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustodyStatus {
    Locked,
    Released,
    Disputed,
    Expired,
}

/// Funds held for an [`apicash_shared::Order`] until release or dispute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Custody {
    pub id: Uuid,
    pub order_id: Uuid,
    /// Principal locked on-chain / in custody (same unit as order).
    pub amount: Money,
    pub status: CustodyStatus,
    pub locked_at: DateTime<Utc>,
    /// Policy horizon (e.g. auto-release or dispute window end).
    pub expected_release_at: DateTime<Utc>,
    pub actual_release_at: Option<DateTime<Utc>>,
    /// Total yield accrued while locked (recorded separately from principal; split on release via [`crate::models::YieldDistribution`]).
    pub yield_earned: Option<Money>,
    /// Contrato Soroban de escrow (Stellar `C...`), se lock on-chain.
    #[serde(default)]
    pub soroban_escrow_contract_id: Option<String>,
    /// Whether Soroban operations were mocked (bridge fallback) at lock time.
    #[serde(default)]
    pub soroban_is_mock: bool,
    #[serde(default)]
    pub soroban_lock_tx_hash: Option<String>,
    #[serde(default)]
    pub soroban_release_tx_hash: Option<String>,
}
