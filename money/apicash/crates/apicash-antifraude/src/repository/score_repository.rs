//! Persistence boundary for scores and dispute counts (in-memory default; swap for SQLx later).

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::AntiFraudeError;
use crate::score::{OnRampDecision, RiskFactor, RiskLevel, UserScore};

#[async_trait]
pub trait ScoreRepository: Send + Sync {
    async fn save_score(&self, score: &UserScore) -> Result<(), AntiFraudeError>;
    async fn open_dispute_count(&self, user_id: Uuid) -> Result<u32, AntiFraudeError>;
    /// Painel admin / filtros de risco.
    async fn list_all_scores(&self) -> Result<Vec<UserScore>, AntiFraudeError>;
}

/// Thread-safe in-memory store for tests and local development.
pub struct InMemoryScoreRepository {
    scores: RwLock<HashMap<Uuid, UserScore>>,
    disputes: RwLock<HashMap<Uuid, u32>>,
}

impl InMemoryScoreRepository {
    pub fn new() -> Self {
        Self {
            scores: RwLock::new(HashMap::new()),
            disputes: RwLock::new(HashMap::new()),
        }
    }

    /// Test helper: seed dispute count for a user.
    pub async fn seed_disputes(&self, user_id: Uuid, count: u32) {
        self.disputes.write().await.insert(user_id, count);
    }
}

impl Default for InMemoryScoreRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ScoreRepository for InMemoryScoreRepository {
    async fn save_score(&self, score: &UserScore) -> Result<(), AntiFraudeError> {
        self.scores
            .write()
            .await
            .insert(score.user_id, score.clone());
        Ok(())
    }

    async fn open_dispute_count(&self, user_id: Uuid) -> Result<u32, AntiFraudeError> {
        Ok(self
            .disputes
            .read()
            .await
            .get(&user_id)
            .copied()
            .unwrap_or(0))
    }

    async fn list_all_scores(&self) -> Result<Vec<UserScore>, AntiFraudeError> {
        Ok(self.scores.read().await.values().cloned().collect())
    }
}

pub struct PostgresScoreRepository {
    pool: PgPool,
}

impl PostgresScoreRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(
        user_id: Uuid,
        score: i32,
        risk_level: String,
        factors: serde_json::Value,
        last_updated: DateTime<Utc>,
        decision: String,
    ) -> Result<UserScore, AntiFraudeError> {
        if score < 0 {
            return Err(AntiFraudeError::Repository(format!(
                "negative score for user {user_id}"
            )));
        }
        let factors: Vec<RiskFactor> = serde_json::from_value(factors)
            .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        Ok(UserScore {
            user_id,
            score: score as u32,
            risk_level: risk_level_from_str(&risk_level)?,
            factors,
            last_updated,
            decision: decision_from_str(&decision)?,
        })
    }
}

#[async_trait]
impl ScoreRepository for PostgresScoreRepository {
    async fn save_score(&self, score: &UserScore) -> Result<(), AntiFraudeError> {
        let score_i32 = i32::try_from(score.score)
            .map_err(|_| AntiFraudeError::Repository("score overflow".into()))?;
        let factors = serde_json::to_value(&score.factors)
            .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        sqlx::query(
            r#"
            INSERT INTO user_scores (user_id, score, risk_level, factors, last_updated, decision)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id) DO UPDATE SET
                score = EXCLUDED.score,
                risk_level = EXCLUDED.risk_level,
                factors = EXCLUDED.factors,
                last_updated = EXCLUDED.last_updated,
                decision = EXCLUDED.decision
            "#,
        )
        .bind(score.user_id)
        .bind(score_i32)
        .bind(risk_level_to_str(score.risk_level))
        .bind(factors)
        .bind(score.last_updated)
        .bind(decision_to_str(score.decision))
        .execute(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        Ok(())
    }

    async fn open_dispute_count(&self, user_id: Uuid) -> Result<u32, AntiFraudeError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*)::BIGINT AS n
            FROM disputes
            WHERE opened_by_user_id = $1 AND status IN ('open', 'under_review')
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        let n: i64 = row
            .try_get("n")
            .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        u32::try_from(n).map_err(|_| AntiFraudeError::Repository("dispute count overflow".into()))
    }

    async fn list_all_scores(&self) -> Result<Vec<UserScore>, AntiFraudeError> {
        let rows = sqlx::query(
            r#"
            SELECT user_id, score, risk_level, factors, last_updated, decision
            FROM user_scores
            ORDER BY last_updated DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            out.push(Self::map_row(
                r.try_get("user_id")
                    .map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                r.try_get("score")
                    .map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                r.try_get("risk_level")
                    .map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                r.try_get("factors")
                    .map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                r.try_get("last_updated")
                    .map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                r.try_get("decision")
                    .map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
            )?);
        }
        Ok(out)
    }
}

fn risk_level_to_str(r: RiskLevel) -> &'static str {
    match r {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
        RiskLevel::Critical => "critical",
    }
}

fn risk_level_from_str(s: &str) -> Result<RiskLevel, AntiFraudeError> {
    match s {
        "low" => Ok(RiskLevel::Low),
        "medium" => Ok(RiskLevel::Medium),
        "high" => Ok(RiskLevel::High),
        "critical" => Ok(RiskLevel::Critical),
        _ => Err(AntiFraudeError::Repository(format!(
            "unknown risk level: {s}"
        ))),
    }
}

fn decision_to_str(d: OnRampDecision) -> &'static str {
    match d {
        OnRampDecision::Approve => "approve",
        OnRampDecision::Review => "review",
        OnRampDecision::Block => "block",
    }
}

fn decision_from_str(s: &str) -> Result<OnRampDecision, AntiFraudeError> {
    match s {
        "approve" => Ok(OnRampDecision::Approve),
        "review" => Ok(OnRampDecision::Review),
        "block" => Ok(OnRampDecision::Block),
        _ => Err(AntiFraudeError::Repository(format!(
            "unknown decision: {s}"
        ))),
    }
}
