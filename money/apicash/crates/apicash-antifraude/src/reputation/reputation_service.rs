//! Service to compute `UserReputation` from stored data.

use std::sync::Arc;
use uuid::Uuid;

use crate::repository::ScoreRepository;
use super::reputation_model::UserReputation;

/// Computes user reputation from score history + order stats.
pub struct ReputationService {
    score_repo: Arc<dyn ScoreRepository>,
}

impl ReputationService {
    pub fn new(score_repo: Arc<dyn ScoreRepository>) -> Self {
        Self { score_repo }
    }

    /// Build reputation for `user_id`.
    ///
    /// `completed` / `dispute_count` come from the order repository (caller's responsibility).
    /// `kyc_approved` comes from the user profile.
    pub async fn compute(
        &self,
        user_id: Uuid,
        completed: u32,
        dispute_count: u32,
        kyc_approved: bool,
    ) -> Result<UserReputation, String> {
        let score = self
            .score_repo
            .get_by_user_id(user_id)
            .await
            .map_err(|e| e.to_string())?
            .map(|s| s.score)
            .unwrap_or(0);

        Ok(UserReputation::new(user_id, score, completed, dispute_count, kyc_approved))
    }
}
