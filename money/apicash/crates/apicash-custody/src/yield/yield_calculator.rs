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

#[cfg(test)]
mod tests {
    use super::*;

    fn calc() -> YieldCalculator {
        YieldCalculator::default() // 0.5% per month
    }

    #[test]
    fn zero_days_yields_zero() {
        let y = calc().accrued_yield(Money::new(Decimal::from(100u32)), 0).unwrap();
        assert_eq!(y.decimal(), Decimal::ZERO);
    }

    #[test]
    fn negative_days_is_error() {
        assert!(calc().accrued_yield(Money::new(Decimal::from(100u32)), -1).is_err());
    }

    #[test]
    fn full_month_yields_half_percent() {
        // R$100 for 30 days at 0.5%/month → R$0.50
        let y = calc().accrued_yield(Money::new(Decimal::from(100u32)), 30).unwrap();
        let expected = Decimal::from(100u32) * Decimal::new(5, 3); // 100 * 0.005
        assert!((y.decimal() - expected).abs() < Decimal::new(1, 6));
    }

    #[test]
    fn half_month_yields_proportionally() {
        // R$1000 for 15 days → (1000 * 0.005 / 30) * 15 = R$2.50
        let y = calc().accrued_yield(Money::new(Decimal::from(1000u32)), 15).unwrap();
        let expected = Decimal::new(250, 2); // 2.50
        assert!((y.decimal() - expected).abs() < Decimal::new(1, 6));
    }

    #[test]
    fn zero_principal_yields_zero() {
        let y = calc().accrued_yield(Money::ZERO, 30).unwrap();
        assert_eq!(y.decimal(), Decimal::ZERO);
    }

    #[test]
    fn custom_rate_calculates_correctly() {
        // 3% per month on R$1000 for 30 days → R$30
        let custom = YieldCalculator { monthly_rate: Decimal::new(3, 2) };
        let y = custom.accrued_yield(Money::new(Decimal::from(1000u32)), 30).unwrap();
        let expected = Decimal::from(30u32);
        assert!((y.decimal() - expected).abs() < Decimal::new(1, 6));
    }
}
