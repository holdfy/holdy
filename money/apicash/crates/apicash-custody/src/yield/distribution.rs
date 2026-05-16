//! Split accrued yield using the 70 / 10 / 20 policy ([`apicash_shared::YIELD_DISTRIBUTION_PERCENT`]).
//!
//! On-chain, a **Soroban** escrow contract should distribute minted interest or claimable balances with the same ratios.

use apicash_shared::Money;
use rust_decimal::Decimal;

use crate::errors::CustodyError;
use crate::models::YieldDistribution;

/// Split the **yield pool** (not principal) across seller, buyer cashback, and platform.
pub fn split_yield_pool(pool: Money) -> Result<YieldDistribution, CustodyError> {
    let p = pool.decimal();
    let seller = p
        .checked_mul(YieldDistribution::ratio_seller())
        .ok_or(CustodyError::Arithmetic)?;
    let buyer = p
        .checked_mul(YieldDistribution::ratio_buyer())
        .ok_or(CustodyError::Arithmetic)?;
    let platform = p
        .checked_mul(YieldDistribution::ratio_platform())
        .ok_or(CustodyError::Arithmetic)?;

    // Rounding: ensure sum matches pool by assigning platform the residual.
    let sum_sb = seller.checked_add(buyer).ok_or(CustodyError::Arithmetic)?;
    let platform_adj = p.checked_sub(sum_sb).unwrap_or(platform);

    Ok(YieldDistribution {
        seller_share: Money::new(seller),
        buyer_cashback: Money::new(buyer),
        platform_share: Money::new(platform_adj),
    })
}

/// Verify three ratios sum to 1.0 (sanity check for config drift).
pub fn ratios_sum_to_one() -> bool {
    let one = YieldDistribution::ratio_seller()
        + YieldDistribution::ratio_buyer()
        + YieldDistribution::ratio_platform();
    one == Decimal::ONE
}
