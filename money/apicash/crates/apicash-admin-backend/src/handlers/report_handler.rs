//! `GET /admin/users/score` — risco / score.

use axum::{extract::Query, extract::State, Json};

use crate::dto::{UserScoreListResponse, UserScoreQuery, UserScoreRow};
use crate::error::AdminError;
use crate::state::AdminState;

pub async fn list_user_scores(
    State(state): State<AdminState>,
    Query(q): Query<UserScoreQuery>,
) -> Result<Json<UserScoreListResponse>, AdminError> {
    let mut scores = state.antifraude.list_scores().await?;

    if let Some(max) = q.max_score {
        scores.retain(|s| s.score <= max);
    }
    if let Some(min_risk) = q.min_risk {
        scores.retain(|s| risk_rank(s.risk_level) >= risk_rank(min_risk));
    }

    let users = scores
        .into_iter()
        .map(|s| UserScoreRow {
            user_id: s.user_id,
            score: s.score,
            risk_level: s.risk_level,
        })
        .collect();

    Ok(Json(UserScoreListResponse { users }))
}

fn risk_rank(r: apicash_antifraude::RiskLevel) -> u8 {
    use apicash_antifraude::RiskLevel::*;
    match r {
        Low => 0,
        Medium => 1,
        High => 2,
        Critical => 3,
    }
}
