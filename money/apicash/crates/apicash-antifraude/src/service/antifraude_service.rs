//! Orchestrates validators, scoring, and persistence.

use std::sync::Arc;

use crate::repository::score_repository::ScoreRepository;
use crate::score::{ScoreCalculator, UserScore};
use crate::validation::{SefazValidator, SocialValidator};
use apicash_shared::ApiCashError;
use uuid::Uuid;

/// Facade used by `apicash-core` / gateway before Stellar on-ramp.
pub struct AntiFraudeService {
    sefaz: SefazValidator,
    social: SocialValidator,
    repository: Arc<dyn ScoreRepository>,
}

impl AntiFraudeService {
    pub fn new(
        sefaz: SefazValidator,
        social: SocialValidator,
        repository: Arc<dyn ScoreRepository>,
    ) -> Self {
        Self {
            sefaz,
            social,
            repository,
        }
    }

    /// Full scoring pipeline: SEFAZ CPF → social links → dispute history → persist score.
    pub async fn calculate_score(
        &self,
        user_id: Uuid,
        cpf: &str,
        social_links: &[String],
    ) -> Result<UserScore, ApiCashError> {
        let sefaz = self
            .sefaz
            .validate_cpf(cpf)
            .await
            .map_err(ApiCashError::from)?;
        let social_out = self
            .social
            .validate_links(social_links)
            .await
            .map_err(ApiCashError::from)?;

        let snapshots: Vec<_> = social_out.into_iter().filter_map(|r| r.snapshot).collect();

        let open_disputes = self
            .repository
            .open_dispute_count(user_id)
            .await
            .map_err(ApiCashError::from)?;

        let score = ScoreCalculator::build_score(user_id, sefaz.status, &snapshots, open_disputes);

        self.repository
            .save_score(&score)
            .await
            .map_err(ApiCashError::from)?;
        Ok(score)
    }

    /// Todos os scores persistidos (admin / relatórios).
    pub async fn list_scores(&self) -> Result<Vec<UserScore>, ApiCashError> {
        self.repository
            .list_all_scores()
            .await
            .map_err(ApiCashError::from)
    }
}
