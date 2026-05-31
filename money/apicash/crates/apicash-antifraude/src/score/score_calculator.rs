//! Deterministic scoring rules for on-ramp eligibility.
//!
//! Score scale: 0–1000.
//!
//! Signal groups and calibrated weights:
//!   Identity (CPF)       — highest weight; verified identity is the foundation of trust
//!   Social reputation    — old accounts reduce disposable-identity risk
//!   Dispute history      — bilateral: BY user and AGAINST user
//!   Velocity             — transaction count in 24h window
//!   Volume               — BRL amount in 24h window
//!   Structuring (COAF)   — amounts near regulatory reporting thresholds
//!   Account maturity     — platform history and account age

use rust_decimal::Decimal;

use apicash_shared::USER_SCORE_MAX;
use chrono::Utc;
use uuid::Uuid;

use crate::score::behavioral_context::BehavioralContext;
use crate::score::risk_factors::RiskFactor;
use crate::score::user_score::{OnRampDecision, RiskLevel, UserScore};
use crate::validation::{DocumentStatus, DocumentType, SocialAccountSnapshot};

// ─── Identity (CPF) ──────────────────────────────────────────────────────────
pub const POINTS_CPF_REGULAR: i32 = 350;
pub const PENALTY_CPF_IRREGULAR: i32 = -320;
pub const SOCIAL_AGE_THRESHOLD_MONTHS: u32 = 6;
pub const POINTS_SOCIAL_OVER_SIX_MONTHS: i32 = 180;

// ─── Identity (CNPJ) ─────────────────────────────────────────────────────────
pub const POINTS_CNPJ_ATIVA: i32 = 100;
pub const PENALTY_CNPJ_INATIVA: i32 = -200;
/// Company with ≥24 months of existence gets a trust bonus.
pub const CNPJ_AGE_ESTABLISHED_MONTHS: u32 = 24;
/// Company with <6 months is considered high-risk.
pub const CNPJ_AGE_NEW_MONTHS: u32 = 6;
pub const POINTS_CNPJ_ESTABLISHED: i32 = 50;
pub const PENALTY_CNPJ_NEW: i32 = -150;

// ─── Dispute history ─────────────────────────────────────────────────────────
pub const PENALTY_PER_OPEN_DISPUTE: i32 = -110;
pub const PENALTY_PER_COUNTERPARTY_DISPUTE: i32 = -90;
pub const DISPUTE_RATE_THRESHOLD_PCT: f64 = 20.0;
pub const PENALTY_HIGH_DISPUTE_RATE: i32 = -150;

// ─── Velocity (24-hour window) ────────────────────────────────────────────────
pub const VELOCITY_WINDOW_HOURS: u32 = 24;
pub const VELOCITY_HIGH_THRESHOLD: u32 = 5;
pub const VELOCITY_MEDIUM_THRESHOLD: u32 = 3;
pub const PENALTY_VELOCITY_HIGH: i32 = -200;
pub const PENALTY_VELOCITY_MEDIUM: i32 = -80;

// ─── Volume (24-hour window, BRL) ────────────────────────────────────────────
pub const VOLUME_HIGH_BRL: u64 = 20_000;
pub const VOLUME_NEW_ACCOUNT_BRL: u64 = 5_000;
pub const PENALTY_HIGH_VOLUME: i32 = -150;
pub const PENALTY_NEW_ACCOUNT_HIGH_VOLUME: i32 = -120;
pub const NEW_ACCOUNT_AGE_DAYS: u32 = 7;

// ─── Structuring (COAF threshold avoidance) ─────────────────────────────────
// Bands just below R$2.000 and R$10.000 are common structuring signals.
pub const PENALTY_STRUCTURING: i32 = -180;

// ─── Account maturity ────────────────────────────────────────────────────────
pub const MATURITY_ESTABLISHED_TX: u32 = 30;
pub const MATURITY_CLEAN_TX_MIN: u32 = 5;
pub const POINTS_ESTABLISHED_USER: i32 = 100;
pub const POINTS_CLEAN_HISTORY: i32 = 60;
pub const PENALTY_FIRST_TX: i32 = 0;
pub const NEW_ACCOUNT_HIGH_VALUE_BRL: u64 = 500;
pub const PENALTY_NEW_ACCOUNT_HIGH_VALUE: i32 = -120;

// ─── Value anomaly ───────────────────────────────────────────────────────────
pub const VALUE_ANOMALY_RATIO: u32 = 300; // 3× average → anomaly
pub const PENALTY_VALUE_ANOMALY: i32 = -100;

// ─── Decision / risk thresholds ──────────────────────────────────────────────
pub const THRESHOLD_LOW_MIN: u32 = 750;
pub const THRESHOLD_MEDIUM_MIN: u32 = 500;
pub const THRESHOLD_HIGH_MIN: u32 = 250;
pub const DECISION_APPROVE_MIN: u32 = 650;
/// Review threshold fixo (300): CPF válido com 1 penalidade menor ainda passa para Review.
/// Anterior era POINTS_CPF_REGULAR (350), o que bloqueava qualquer penalidade em usuário novo.
pub const DECISION_REVIEW_MIN: u32 = 300;
/// ValueAnomaly só aplica com histórico mínimo de 5 transações — evita falso positivo em usuários novos.
pub const VALUE_ANOMALY_MIN_TX: u32 = 5;

pub struct ScoreCalculator;

impl ScoreCalculator {
    /// Stable recommendation string for API consumers and SIEM rules.
    #[must_use]
    pub fn get_risk_recommendation(score: &UserScore) -> &'static str {
        score.get_risk_recommendation()
    }

    /// Build a bounded 0–1000 score from identity, social, and behavioral signals.
    pub fn build_score(
        user_id: Uuid,
        doc_type: DocumentType,
        doc_status: DocumentStatus,
        social: &[SocialAccountSnapshot],
        ctx: &BehavioralContext,
    ) -> UserScore {
        let mut raw: i32 = 0;
        let mut factors: Vec<RiskFactor> = Vec::new();

        // ── Identity ────────────────────────────────────────────────────────
        match doc_type {
            DocumentType::Cpf => match doc_status {
                DocumentStatus::Valid => {
                    raw += POINTS_CPF_REGULAR;
                    factors.push(RiskFactor::SefazStatus { weight: POINTS_CPF_REGULAR });
                }
                DocumentStatus::Invalid => {
                    raw += PENALTY_CPF_IRREGULAR;
                    factors.push(RiskFactor::SefazStatus { weight: PENALTY_CPF_IRREGULAR });
                }
                DocumentStatus::Unknown => {
                    factors.push(RiskFactor::Other {
                        code: "document_unknown".into(),
                        weight: 0,
                    });
                }
            },
            DocumentType::Cnpj => {
                let id_weight = match doc_status {
                    DocumentStatus::Valid => POINTS_CNPJ_ATIVA,
                    DocumentStatus::Invalid => PENALTY_CNPJ_INATIVA,
                    DocumentStatus::Unknown => 0,
                };
                if id_weight != 0 {
                    raw += id_weight;
                }
                factors.push(RiskFactor::CnpjStatus {
                    active: doc_status == DocumentStatus::Valid,
                    weight: id_weight,
                });

                if let Some(months) = ctx.company_age_months {
                    let age_weight = if months >= CNPJ_AGE_ESTABLISHED_MONTHS {
                        POINTS_CNPJ_ESTABLISHED
                    } else if months < CNPJ_AGE_NEW_MONTHS {
                        PENALTY_CNPJ_NEW
                    } else {
                        0
                    };
                    if age_weight != 0 {
                        raw += age_weight;
                        factors.push(RiskFactor::CompanyAge { months, weight: age_weight });
                    }
                }
            }
        }

        if let Some(s) = social.iter().find(|s| s.estimated_age_months >= SOCIAL_AGE_THRESHOLD_MONTHS) {
            raw += POINTS_SOCIAL_OVER_SIX_MONTHS;
            factors.push(RiskFactor::SocialAccountAge {
                platform: s.platform.clone(),
                months: s.estimated_age_months,
                weight: POINTS_SOCIAL_OVER_SIX_MONTHS,
            });
        }

        // ── Dispute history (BY user) ────────────────────────────────────
        if ctx.open_dispute_count > 0 {
            let w = PENALTY_PER_OPEN_DISPUTE.saturating_mul(ctx.open_dispute_count as i32);
            raw += w;
            factors.push(RiskFactor::DisputeHistory {
                open_disputes: ctx.open_dispute_count,
                weight: w,
            });
        }

        // ── Counterparty disputes (AGAINST user) ────────────────────────
        if ctx.disputes_as_counterparty > 0 {
            let w = PENALTY_PER_COUNTERPARTY_DISPUTE
                .saturating_mul(ctx.disputes_as_counterparty as i32);
            raw += w;
            factors.push(RiskFactor::CounterpartyDisputes {
                count: ctx.disputes_as_counterparty,
                weight: w,
            });
        }

        // ── Dispute rate ─────────────────────────────────────────────────
        let dispute_rate_pct = (ctx.dispute_rate * 100.0) as u32;
        if ctx.tx_count_total > 0 && ctx.dispute_rate * 100.0 > DISPUTE_RATE_THRESHOLD_PCT {
            raw += PENALTY_HIGH_DISPUTE_RATE;
            factors.push(RiskFactor::DisputeRate {
                rate_pct: dispute_rate_pct,
                weight: PENALTY_HIGH_DISPUTE_RATE,
            });
        }

        // ── Velocity (24h) ───────────────────────────────────────────────
        let velocity_weight = if ctx.tx_count_24h > VELOCITY_HIGH_THRESHOLD {
            Some(PENALTY_VELOCITY_HIGH)
        } else if ctx.tx_count_24h >= VELOCITY_MEDIUM_THRESHOLD {
            Some(PENALTY_VELOCITY_MEDIUM)
        } else {
            None
        };
        if let Some(w) = velocity_weight {
            raw += w;
            factors.push(RiskFactor::VelocityCheck {
                tx_count: ctx.tx_count_24h,
                window_hours: VELOCITY_WINDOW_HOURS,
                weight: w,
            });
        }

        // ── High volume (24h) ────────────────────────────────────────────
        let volume_brl_u64 = ctx.tx_volume_24h_brl.to_u64_saturating();
        if volume_brl_u64 >= VOLUME_HIGH_BRL {
            raw += PENALTY_HIGH_VOLUME;
            factors.push(RiskFactor::HighVolume {
                amount_brl: ctx.tx_volume_24h_brl.to_string(),
                window_hours: VELOCITY_WINDOW_HOURS,
                weight: PENALTY_HIGH_VOLUME,
            });
        } else if volume_brl_u64 >= VOLUME_NEW_ACCOUNT_BRL
            && ctx.account_age_days <= NEW_ACCOUNT_AGE_DAYS
        {
            raw += PENALTY_NEW_ACCOUNT_HIGH_VOLUME;
            factors.push(RiskFactor::HighVolume {
                amount_brl: ctx.tx_volume_24h_brl.to_string(),
                window_hours: VELOCITY_WINDOW_HOURS,
                weight: PENALTY_NEW_ACCOUNT_HIGH_VOLUME,
            });
        }

        // ── Structuring ──────────────────────────────────────────────────
        if let Some(ref amt) = ctx.current_tx_amount {
            if is_structuring_amount(amt) {
                raw += PENALTY_STRUCTURING;
                factors.push(RiskFactor::Structuring {
                    amount_brl: amt.to_string(),
                    weight: PENALTY_STRUCTURING,
                });
            }

            // Value anomaly: current amount ≥ 3× historical average.
            // Requer ao menos VALUE_ANOMALY_MIN_TX transações históricas — evita falso
            // positivo em usuário novo cujo 2º pedido seja maior que o 1º.
            if ctx.tx_count_total >= VALUE_ANOMALY_MIN_TX {
                if let Some(ref avg) = ctx.avg_tx_value {
                    if !avg.is_zero() {
                        let ratio_pct = (amt / avg * Decimal::from(100u32))
                            .to_u64_saturating() as u32;
                        if ratio_pct >= VALUE_ANOMALY_RATIO {
                            raw += PENALTY_VALUE_ANOMALY;
                            factors.push(RiskFactor::ValueAnomaly {
                                ratio_pct,
                                weight: PENALTY_VALUE_ANOMALY,
                            });
                        }
                    }
                }
            }
        }

        // ── Account maturity ─────────────────────────────────────────────
        let maturity_weight = if ctx.tx_count_total == 0 {
            // First-ever transaction
            let w = PENALTY_FIRST_TX;
            // Extra penalty: new account + high value
            let extra = ctx
                .current_tx_amount
                .as_ref()
                .filter(|a| a.to_u64_saturating() >= NEW_ACCOUNT_HIGH_VALUE_BRL)
                .map(|_| PENALTY_NEW_ACCOUNT_HIGH_VALUE)
                .unwrap_or(0);
            w + extra
        } else if ctx.tx_count_total >= MATURITY_ESTABLISHED_TX
            && ctx.open_dispute_count == 0
            && ctx.disputes_as_counterparty == 0
        {
            POINTS_ESTABLISHED_USER
        } else if ctx.tx_count_total >= MATURITY_CLEAN_TX_MIN && ctx.open_dispute_count == 0 {
            POINTS_CLEAN_HISTORY
        } else {
            0
        };

        if maturity_weight != 0 {
            raw += maturity_weight;
            factors.push(RiskFactor::AccountMaturity {
                tx_count: ctx.tx_count_total,
                age_days: ctx.account_age_days,
                weight: maturity_weight,
            });
        }

        let score = raw.clamp(0, USER_SCORE_MAX as i32) as u32;
        let risk_level = risk_level_from_score(score);
        let decision = decision_from_score(score, risk_level);

        UserScore {
            user_id,
            score,
            risk_level,
            factors,
            last_updated: Utc::now(),
            decision,
        }
    }
}

/// Returns `true` when an amount falls in a known structuring band.
///
/// Brazilian COAF reporting thresholds are R$2.000 (individual) and R$10.000 (corporate).
/// Amounts just below these values (within 10%) are a common avoidance pattern.
fn is_structuring_amount(amount: &Decimal) -> bool {
    let brl = amount.to_u64_saturating();
    (1_800..2_000).contains(&brl) || (9_000..10_000).contains(&brl)
}

fn risk_level_from_score(score: u32) -> RiskLevel {
    if score >= THRESHOLD_LOW_MIN {
        RiskLevel::Low
    } else if score >= THRESHOLD_MEDIUM_MIN {
        RiskLevel::Medium
    } else if score >= THRESHOLD_HIGH_MIN {
        RiskLevel::High
    } else {
        RiskLevel::Critical
    }
}

fn decision_from_score(score: u32, risk: RiskLevel) -> OnRampDecision {
    match risk {
        RiskLevel::Critical => OnRampDecision::Block,
        _ if score >= DECISION_APPROVE_MIN => OnRampDecision::Approve,
        _ if score >= DECISION_REVIEW_MIN => OnRampDecision::Review,
        _ => OnRampDecision::Block,
    }
}

trait ToU64Saturating {
    fn to_u64_saturating(&self) -> u64;
}

impl ToU64Saturating for Decimal {
    fn to_u64_saturating(&self) -> u64 {
        if self.is_sign_negative() {
            return 0;
        }
        use rust_decimal::prelude::ToPrimitive;
        self.to_u64().unwrap_or(u64::MAX)
    }
}
