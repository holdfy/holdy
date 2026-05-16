//! Deterministic scoring rules for on-ramp eligibility.
//!
//! Pesos calibrados para produção (escala 0–1000):
//! - **Identidade fiscal (CPF)** — maior peso: cadastro regular é pré-requisito forte para confiança.
//! - **Reputação social** — contas antigas reduzem risco de contas descartáveis.
//! - **Histórico de disputas** — penalidade forte e linear por litígio aberto.

use apicash_shared::USER_SCORE_MAX;

use crate::score::risk_factors::RiskFactor;
use crate::score::user_score::{OnRampDecision, RiskLevel, UserScore};
use crate::validation::{SefazPersonStatus, SocialAccountSnapshot};
use chrono::Utc;
use uuid::Uuid;

/// Confiança base quando o CPF está com situação **Regular** (Receita/SEFAZ-style).
/// ~35% do teto de score; identidade verificável é o pilar do modelo.
pub const POINTS_CPF_REGULAR: i32 = 350;
/// Penalidade severa quando o CPF está **Irregular** — risco de fraude ou identidade inválida.
pub const PENALTY_CPF_IRREGULAR: i32 = -320;
/// Bónus quando existe pelo menos uma rede social com idade ≥ limiar (conta estabelecida).
pub const POINTS_SOCIAL_OVER_SIX_MONTHS: i32 = 180;
/// Penalidade por **cada** disputa em aberto (chargeback / litígio).
pub const PENALTY_PER_OPEN_DISPUTE: i32 = 110;
/// Idade mínima de conta social (meses) para o bónus.
pub const SOCIAL_AGE_THRESHOLD_MONTHS: u32 = 6;

/// Limiares de [`RiskLevel`] — buckets para relatórios e dashboards.
pub const THRESHOLD_LOW_MIN: u32 = 750;
pub const THRESHOLD_MEDIUM_MIN: u32 = 500;
pub const THRESHOLD_HIGH_MIN: u32 = 250;

/// Limiares para [`OnRampDecision`] (automático vs revisão manual vs bloqueio).
pub const DECISION_APPROVE_MIN: u32 = 650;
/// Deve ser ≤ [`POINTS_CPF_REGULAR`]: só com identidade fiscal *Regular* já entra em *Review*
/// (PIX / on-ramp permitido onde só `Block` é recusado). Sem isto, fluxos só com CPF válido —
/// como o WhatsApp sem `social_links` — ficam sempre bloqueados.
pub const DECISION_REVIEW_MIN: u32 = POINTS_CPF_REGULAR as u32;

pub struct ScoreCalculator;

impl ScoreCalculator {
    /// Retorna recomendação textual estável para integrações (APIs, SIEM, regras).
    #[must_use]
    pub fn get_risk_recommendation(score: &UserScore) -> &'static str {
        score.get_risk_recommendation()
    }

    /// Combine validator outputs into a bounded 0–1000 score and decision.
    pub fn build_score(
        user_id: Uuid,
        sefaz: SefazPersonStatus,
        social: &[SocialAccountSnapshot],
        open_dispute_count: u32,
    ) -> UserScore {
        let mut raw: i32 = 0;
        let mut factors: Vec<RiskFactor> = Vec::new();

        match sefaz {
            SefazPersonStatus::Regular => {
                raw += POINTS_CPF_REGULAR;
                factors.push(RiskFactor::SefazStatus {
                    weight: POINTS_CPF_REGULAR,
                });
            }
            SefazPersonStatus::Irregular => {
                raw += PENALTY_CPF_IRREGULAR;
                factors.push(RiskFactor::SefazStatus {
                    weight: PENALTY_CPF_IRREGULAR,
                });
            }
            SefazPersonStatus::Unknown => {
                factors.push(RiskFactor::Other {
                    code: "sefaz_unknown".into(),
                    weight: 0,
                });
            }
        }

        if let Some(s) = social
            .iter()
            .find(|s| s.estimated_age_months >= SOCIAL_AGE_THRESHOLD_MONTHS)
        {
            raw += POINTS_SOCIAL_OVER_SIX_MONTHS;
            factors.push(RiskFactor::SocialAccountAge {
                platform: s.platform.clone(),
                months: s.estimated_age_months,
                weight: POINTS_SOCIAL_OVER_SIX_MONTHS,
            });
        }

        if open_dispute_count > 0 {
            let penalty = PENALTY_PER_OPEN_DISPUTE.saturating_mul(open_dispute_count as i32);
            raw -= penalty;
            factors.push(RiskFactor::DisputeHistory {
                open_disputes: open_dispute_count,
                weight: -penalty,
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
