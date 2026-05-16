//! Corpo de `POST /risk/score` — pré-cálculo de score (fatores completos).

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RiskScoreRequest {
    pub user_id: Uuid,
    pub cpf: String,
    #[serde(default)]
    pub social_links: Vec<String>,
}

impl RiskScoreRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        let digits: String = self.cpf.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() != 11 {
            return Err("cpf must have 11 digits");
        }
        Ok(())
    }
}
