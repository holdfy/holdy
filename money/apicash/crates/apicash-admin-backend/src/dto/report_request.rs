//! Query params para relatórios.

use apicash_antifraude::RiskLevel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct YieldReportQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct YieldReportResponse {
    pub total_yield_minor: String,
    pub custody_count: usize,
    pub released_count: usize,
    pub period_from: Option<DateTime<Utc>>,
    pub period_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UserScoreQuery {
    /// Score máximo (inclusivo) para filtrar “baixo score”.
    pub max_score: Option<u32>,
    /// Nível de risco mínimo (ex.: high).
    pub min_risk: Option<RiskLevel>,
}

#[derive(Debug, Serialize)]
pub struct UserScoreListResponse {
    pub users: Vec<UserScoreRow>,
}

#[derive(Debug, Serialize)]
pub struct UserScoreRow {
    pub user_id: uuid::Uuid,
    pub score: u32,
    pub risk_level: RiskLevel,
}
