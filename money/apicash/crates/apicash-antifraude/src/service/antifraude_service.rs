//! Orchestrates validators, scoring, and persistence.

use std::sync::Arc;

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::repository::score_repository::ScoreRepository;
use crate::score::{BehavioralContext, ScoreCalculator, UserScore};
use crate::validation::{DocumentType, DocumentValidator, SocialValidator};
use apicash_shared::ApiCashError;

/// Facade used by `apicash-core` before allowing a PIX on-ramp.
pub struct AntiFraudeService {
    document_validator: Arc<dyn DocumentValidator>,
    social: SocialValidator,
    repository: Arc<dyn ScoreRepository>,
}

impl AntiFraudeService {
    pub fn new(
        document_validator: Arc<dyn DocumentValidator>,
        social: SocialValidator,
        repository: Arc<dyn ScoreRepository>,
    ) -> Self {
        Self { document_validator, social, repository }
    }

    /// Full scoring pipeline:
    ///   Document validation → social links → behavioral context → score → persist → return.
    ///
    /// `current_tx_amount` — the BRL amount of the transaction being evaluated.
    /// Pass `None` for standalone score checks (e.g. `POST /risk/score`).
    pub async fn calculate_score(
        &self,
        user_id: Uuid,
        cpf: &str,
        social_links: &[String],
        current_tx_amount: Option<Decimal>,
    ) -> Result<UserScore, ApiCashError> {
        // ── Identity validation ───────────────────────────────────────────
        let digits: String = cpf.chars().filter(|c| c.is_ascii_digit()).collect();
        let doc_type = if digits.len() == 14 {
            DocumentType::Cnpj
        } else {
            DocumentType::Cpf
        };
        let doc_status = self
            .document_validator
            .validate(cpf, doc_type)
            .await
            .map_err(ApiCashError::from)?;

        let social_out = self
            .social
            .validate_links(social_links)
            .await
            .map_err(ApiCashError::from)?;

        let snapshots: Vec<_> = social_out.into_iter().filter_map(|r| r.snapshot).collect();

        // ── Behavioral context (parallel repository queries) ──────────────
        let (
            open_disputes,
            counterparty_disputes,
            tx_count_24h,
            tx_volume_24h,
            tx_count_total,
            avg_tx_value,
            account_age_days,
        ) = tokio::try_join!(
            self.repository.open_dispute_count(user_id),
            self.repository.disputes_as_counterparty(user_id),
            self.repository.transaction_count(user_id, 24),
            self.repository.transaction_volume(user_id, 24),
            self.repository.total_transaction_count(user_id),
            self.repository.average_transaction_value(user_id),
            self.repository.account_age_days(user_id),
        )
        .map_err(ApiCashError::from)?;

        let dispute_rate = if tx_count_total > 0 {
            (open_disputes + counterparty_disputes) as f64 / tx_count_total as f64
        } else {
            0.0
        };

        let ctx = BehavioralContext {
            open_dispute_count: open_disputes,
            disputes_as_counterparty: counterparty_disputes,
            dispute_rate,
            tx_count_24h,
            tx_count_total,
            tx_volume_24h_brl: tx_volume_24h,
            avg_tx_value,
            account_age_days,
            current_tx_amount,
            company_age_months: None, // populated when SEFAZ lookup is implemented
        };

        let score = ScoreCalculator::build_score(user_id, doc_type, doc_status, &snapshots, &ctx);

        self.repository
            .save_score(&score)
            .await
            .map_err(ApiCashError::from)?;

        Ok(score)
    }

    /// All persisted scores (admin / reports).
    pub async fn list_scores(&self) -> Result<Vec<UserScore>, ApiCashError> {
        self.repository
            .list_all_scores()
            .await
            .map_err(ApiCashError::from)
    }

    /// Latest persisted score for a specific user (None if never scored).
    pub async fn get_latest_score(&self, user_id: Uuid) -> Result<Option<UserScore>, ApiCashError> {
        self.repository
            .get_by_user_id(user_id)
            .await
            .map_err(ApiCashError::from)
    }
}
