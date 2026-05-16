//! Accrual of yield on locked principal using a configurable **monthly** rate, applied via a simple daily factor.
//!
//! This is intentionally off-chain today; the same formula can be mirrored in **Soroban** contract logic or
//! fed from oracle rates on-chain.

use apicash_shared::Money;
use rust_decimal::Decimal;

use crate::errors::CustodyError;

/// Configurable yield engine (default: 0.5% per month → daily = monthly / 30).
#[derive(Debug, Clone)]
pub struct YieldCalculator {
    /// Monthly rate as decimal fraction, e.g. `0.005` = 0.5% per month.
    pub monthly_rate: Decimal,
}

impl Default for YieldCalculator {
    fn default() -> Self {
        Self {
            // 0.005 = 0.5% per month
            monthly_rate: Decimal::new(5, 3),
        }
    }
}

impl YieldCalculator {
    /// Simple daily accrual: `principal * (monthly_rate / 30) * days` (no intra-day compounding).
    pub fn accrued_yield(&self, principal: Money, days: i64) -> Result<Money, CustodyError> {
        if days < 0 {
            return Err(CustodyError::YieldCalculation(
                "days cannot be negative".into(),
            ));
        }
        if days == 0 {
            return Ok(Money::ZERO);
        }
        let principal_dec = principal.decimal();
        let daily_rate = self
            .monthly_rate
            .checked_div(Decimal::from(30u8))
            .ok_or(CustodyError::Arithmetic)?;
        let accrual = principal_dec
            .checked_mul(daily_rate)
            .and_then(|x| x.checked_mul(Decimal::from(days)))
            .ok_or(CustodyError::Arithmetic)?;
        Ok(Money::new(accrual))
    }
}
