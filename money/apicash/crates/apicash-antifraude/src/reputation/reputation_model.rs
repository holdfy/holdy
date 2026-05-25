//! Domain types for the reputation and seal system.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trust seal awarded to a user based on history + antifraude score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReputationSeal {
    /// KYC approved + ≥5 completed transactions + zero disputes.
    Verified,
    /// KYC approved + score ≥800 + ≥20 completed transactions + dispute rate <5%.
    Premium,
    /// KYC approved + score ≥900 + ≥50 completed transactions + dispute rate <2%.
    Authenticated,
}

impl ReputationSeal {
    pub fn label(&self) -> &'static str {
        match self {
            ReputationSeal::Verified => "Verificado",
            ReputationSeal::Premium => "Premium",
            ReputationSeal::Authenticated => "Autenticado",
        }
    }

    pub fn badge_color(&self) -> &'static str {
        match self {
            ReputationSeal::Verified => "blue",
            ReputationSeal::Premium => "gold",
            ReputationSeal::Authenticated => "green",
        }
    }
}

/// Full reputation snapshot for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReputation {
    pub user_id: Uuid,
    /// Antifraude score (0–1000).
    pub score: u32,
    /// Total orders completed (as buyer or seller).
    pub completed_transactions: u32,
    /// Orders with disputes raised (as a fraction of completed).
    pub dispute_rate: Decimal,
    pub seal: Option<ReputationSeal>,
    pub kyc_approved: bool,
    pub computed_at: DateTime<Utc>,
}

impl UserReputation {
    /// Compute the highest achievable seal given the current state.
    ///
    /// Seals are independent thresholds evaluated highest-first; the first one that
    /// passes is returned. Verified has zero-tolerance for disputes; Premium and
    /// Authenticated allow increasing dispute rates because volume justifies it.
    pub fn compute_seal(
        score: u32,
        completed: u32,
        dispute_rate: Decimal,
        kyc_approved: bool,
    ) -> Option<ReputationSeal> {
        if !kyc_approved {
            return None;
        }

        // Authenticated: highest bar — score ≥900, ≥50 txns, dispute_rate <2%
        let two_pct = Decimal::new(2, 2); // 0.02
        if score >= 900 && completed >= 50 && dispute_rate < two_pct {
            return Some(ReputationSeal::Authenticated);
        }

        // Premium: score ≥800, ≥20 txns, dispute_rate <5%
        let five_pct = Decimal::new(5, 2); // 0.05
        if score >= 800 && completed >= 20 && dispute_rate < five_pct {
            return Some(ReputationSeal::Premium);
        }

        // Verified: ≥5 txns, zero disputes
        if completed >= 5 && dispute_rate == Decimal::ZERO {
            return Some(ReputationSeal::Verified);
        }

        None
    }

    pub fn new(
        user_id: Uuid,
        score: u32,
        completed_transactions: u32,
        dispute_count: u32,
        kyc_approved: bool,
    ) -> Self {
        let dispute_rate = if completed_transactions > 0 {
            Decimal::from(dispute_count) / Decimal::from(completed_transactions)
        } else {
            Decimal::ZERO
        };
        let seal = Self::compute_seal(score, completed_transactions, dispute_rate, kyc_approved);
        Self {
            user_id,
            score,
            completed_transactions,
            dispute_rate,
            seal,
            kyc_approved,
            computed_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::prelude::FromPrimitive;

    fn d(pct: f64) -> Decimal {
        Decimal::from_f64(pct / 100.0).unwrap_or(Decimal::ZERO)
    }

    #[test]
    fn no_kyc_no_seal() {
        assert_eq!(UserReputation::compute_seal(900, 50, Decimal::ZERO, false), None);
    }

    #[test]
    fn kyc_but_too_few_transactions() {
        assert_eq!(UserReputation::compute_seal(900, 4, Decimal::ZERO, true), None);
    }

    #[test]
    fn kyc_with_disputes_no_seal() {
        assert_eq!(UserReputation::compute_seal(900, 10, d(1.0), true), None);
    }

    #[test]
    fn verified_seal() {
        assert_eq!(
            UserReputation::compute_seal(700, 5, Decimal::ZERO, true),
            Some(ReputationSeal::Verified)
        );
    }

    #[test]
    fn premium_requires_score_800_and_20_transactions() {
        assert_eq!(
            UserReputation::compute_seal(799, 20, Decimal::ZERO, true),
            Some(ReputationSeal::Verified)
        );
        assert_eq!(
            UserReputation::compute_seal(800, 19, Decimal::ZERO, true),
            Some(ReputationSeal::Verified)
        );
        assert_eq!(
            UserReputation::compute_seal(800, 20, Decimal::ZERO, true),
            Some(ReputationSeal::Premium)
        );
    }

    #[test]
    fn authenticated_requires_50_transactions_and_low_dispute_rate() {
        assert_eq!(
            UserReputation::compute_seal(900, 49, Decimal::ZERO, true),
            Some(ReputationSeal::Premium)
        );
        // dispute rate exactly 2% is NOT below threshold
        assert_eq!(
            UserReputation::compute_seal(900, 50, d(2.0), true),
            Some(ReputationSeal::Premium)
        );
        // dispute rate 1.9% is below threshold
        assert_eq!(
            UserReputation::compute_seal(900, 50, d(1.9), true),
            Some(ReputationSeal::Authenticated)
        );
        assert_eq!(
            UserReputation::compute_seal(900, 50, Decimal::ZERO, true),
            Some(ReputationSeal::Authenticated)
        );
    }

    #[test]
    fn new_computes_dispute_rate_correctly() {
        let uid = Uuid::new_v4();
        let rep = UserReputation::new(uid, 900, 50, 1, true);
        // dispute_rate = 1/50 = 0.02 = 2% → not < 2%, so Premium
        assert_eq!(rep.seal, Some(ReputationSeal::Premium));
        assert_eq!(rep.dispute_rate, Decimal::new(2, 2));
    }

    #[test]
    fn new_with_zero_completed_gives_no_seal() {
        let uid = Uuid::new_v4();
        let rep = UserReputation::new(uid, 900, 0, 0, true);
        assert_eq!(rep.seal, None);
        assert_eq!(rep.dispute_rate, Decimal::ZERO);
    }
}
