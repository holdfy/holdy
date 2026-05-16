//! Split of accrued **yield** (not principal) across parties — aligns with shared 70/10/20 policy.
//!
//! Integration note: on Stellar, the Soroban contract should enforce the same split at release.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use apicash_shared::Money;

/// Distribution of the **yield pool** at release: 70% seller, 10% buyer cashback, 20% platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YieldDistribution {
    /// Typically 70% of the yield pool.
    pub seller_share: Money,
    /// Typically 10% buyer cashback from yield.
    pub buyer_cashback: Money,
    /// Typically 20% platform fee from yield.
    pub platform_share: Money,
}

impl YieldDistribution {
    /// Fixed ratios (sum to 1.0) used when splitting the yield pool.
    pub fn ratio_seller() -> Decimal {
        Decimal::new(70, 2) // 0.70
    }

    pub fn ratio_buyer() -> Decimal {
        Decimal::new(10, 2) // 0.10
    }

    pub fn ratio_platform() -> Decimal {
        Decimal::new(20, 2) // 0.20
    }
}
