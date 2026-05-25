//! Persistence boundary for scores and behavioral signals (in-memory default; swap for SQLx).

use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::AntiFraudeError;
use crate::score::{OnRampDecision, RiskFactor, RiskLevel, UserScore};

#[async_trait]
pub trait ScoreRepository: Send + Sync {
    async fn save_score(&self, score: &UserScore) -> Result<(), AntiFraudeError>;
    async fn get_by_user_id(&self, user_id: Uuid) -> Result<Option<UserScore>, AntiFraudeError>;
    async fn list_all_scores(&self) -> Result<Vec<UserScore>, AntiFraudeError>;

    // ── Dispute signals ───────────────────────────────────────────────────
    /// Disputes currently open that were opened BY this user.
    async fn open_dispute_count(&self, user_id: Uuid) -> Result<u32, AntiFraudeError>;
    /// Disputes currently open that were opened AGAINST this user by a counterparty.
    async fn disputes_as_counterparty(&self, user_id: Uuid) -> Result<u32, AntiFraudeError>;

    // ── Transaction behavioral signals ────────────────────────────────────
    /// Number of completed/pending orders in the last `window_hours` hours.
    async fn transaction_count(
        &self,
        user_id: Uuid,
        window_hours: u32,
    ) -> Result<u32, AntiFraudeError>;
    /// Total BRL volume of orders in the last `window_hours` hours.
    async fn transaction_volume(
        &self,
        user_id: Uuid,
        window_hours: u32,
    ) -> Result<Decimal, AntiFraudeError>;
    /// Total number of orders ever created by this user.
    async fn total_transaction_count(&self, user_id: Uuid) -> Result<u32, AntiFraudeError>;
    /// Historical average order value; `None` if the user has no prior orders.
    async fn average_transaction_value(
        &self,
        user_id: Uuid,
    ) -> Result<Option<Decimal>, AntiFraudeError>;
    /// Days since the user's first activity on the platform (proxy for account age).
    async fn account_age_days(&self, user_id: Uuid) -> Result<u32, AntiFraudeError>;
}

// ─── InMemory ────────────────────────────────────────────────────────────────

/// Behavioral seed data stored per-user for in-memory tests.
#[derive(Default, Clone)]
struct UserBehavior {
    disputes_by: u32,
    disputes_against: u32,
    tx_count_24h: u32,
    tx_volume_24h: Decimal,
    tx_count_total: u32,
    avg_tx_value: Option<Decimal>,
    account_age_days: u32,
}

/// Thread-safe in-memory store for tests and local development.
pub struct InMemoryScoreRepository {
    scores: RwLock<HashMap<Uuid, UserScore>>,
    behavior: RwLock<HashMap<Uuid, UserBehavior>>,
}

impl InMemoryScoreRepository {
    pub fn new() -> Self {
        Self {
            scores: RwLock::new(HashMap::new()),
            behavior: RwLock::new(HashMap::new()),
        }
    }

    // ── Test seed helpers ─────────────────────────────────────────────────

    pub async fn seed_disputes(&self, user_id: Uuid, count: u32) {
        self.behavior.write().await.entry(user_id).or_default().disputes_by = count;
    }

    pub async fn seed_counterparty_disputes(&self, user_id: Uuid, count: u32) {
        self.behavior.write().await.entry(user_id).or_default().disputes_against = count;
    }

    pub async fn seed_transactions(
        &self,
        user_id: Uuid,
        count_24h: u32,
        volume_24h: Decimal,
        total: u32,
        avg: Option<Decimal>,
    ) {
        let mut b = self.behavior.write().await;
        let entry = b.entry(user_id).or_default();
        entry.tx_count_24h = count_24h;
        entry.tx_volume_24h = volume_24h;
        entry.tx_count_total = total;
        entry.avg_tx_value = avg;
    }

    pub async fn seed_account_age(&self, user_id: Uuid, days: u32) {
        self.behavior.write().await.entry(user_id).or_default().account_age_days = days;
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
        self.scores.write().await.insert(score.user_id, score.clone());
        Ok(())
    }

    async fn get_by_user_id(&self, user_id: Uuid) -> Result<Option<UserScore>, AntiFraudeError> {
        Ok(self.scores.read().await.get(&user_id).cloned())
    }

    async fn list_all_scores(&self) -> Result<Vec<UserScore>, AntiFraudeError> {
        Ok(self.scores.read().await.values().cloned().collect())
    }

    async fn open_dispute_count(&self, user_id: Uuid) -> Result<u32, AntiFraudeError> {
        Ok(self.behavior.read().await.get(&user_id).map(|b| b.disputes_by).unwrap_or(0))
    }

    async fn disputes_as_counterparty(&self, user_id: Uuid) -> Result<u32, AntiFraudeError> {
        Ok(self.behavior.read().await.get(&user_id).map(|b| b.disputes_against).unwrap_or(0))
    }

    async fn transaction_count(&self, user_id: Uuid, _window_hours: u32) -> Result<u32, AntiFraudeError> {
        Ok(self.behavior.read().await.get(&user_id).map(|b| b.tx_count_24h).unwrap_or(0))
    }

    async fn transaction_volume(&self, user_id: Uuid, _window_hours: u32) -> Result<Decimal, AntiFraudeError> {
        Ok(self.behavior.read().await.get(&user_id).map(|b| b.tx_volume_24h).unwrap_or(Decimal::ZERO))
    }

    async fn total_transaction_count(&self, user_id: Uuid) -> Result<u32, AntiFraudeError> {
        Ok(self.behavior.read().await.get(&user_id).map(|b| b.tx_count_total).unwrap_or(0))
    }

    async fn average_transaction_value(&self, user_id: Uuid) -> Result<Option<Decimal>, AntiFraudeError> {
        Ok(self.behavior.read().await.get(&user_id).and_then(|b| b.avg_tx_value))
    }

    async fn account_age_days(&self, user_id: Uuid) -> Result<u32, AntiFraudeError> {
        Ok(self.behavior.read().await.get(&user_id).map(|b| b.account_age_days).unwrap_or(0))
    }
}

// ─── Postgres ────────────────────────────────────────────────────────────────

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
    async fn get_by_user_id(&self, user_id: Uuid) -> Result<Option<UserScore>, AntiFraudeError> {
        let rows = sqlx::query(
            "SELECT user_id, score, risk_level, factors, last_updated, decision
             FROM user_scores WHERE user_id = $1 LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        rows.map(|r| {
            Self::map_row(
                r.try_get("user_id").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                r.try_get("score").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                r.try_get("risk_level").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                r.try_get("factors").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                r.try_get("last_updated").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                r.try_get("decision").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
            )
        })
        .transpose()
    }

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
                score        = EXCLUDED.score,
                risk_level   = EXCLUDED.risk_level,
                factors      = EXCLUDED.factors,
                last_updated = EXCLUDED.last_updated,
                decision     = EXCLUDED.decision
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

    async fn list_all_scores(&self) -> Result<Vec<UserScore>, AntiFraudeError> {
        let rows = sqlx::query(
            "SELECT user_id, score, risk_level, factors, last_updated, decision
             FROM user_scores ORDER BY last_updated DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                Self::map_row(
                    r.try_get("user_id").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                    r.try_get("score").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                    r.try_get("risk_level").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                    r.try_get("factors").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                    r.try_get("last_updated").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                    r.try_get("decision").map_err(|e| AntiFraudeError::Repository(e.to_string()))?,
                )
            })
            .collect()
    }

    async fn open_dispute_count(&self, user_id: Uuid) -> Result<u32, AntiFraudeError> {
        let row = sqlx::query(
            "SELECT COUNT(*)::BIGINT AS n FROM disputes
             WHERE opened_by_user_id = $1 AND status IN ('open', 'under_review')",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        let n: i64 = row.try_get("n").map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        u32::try_from(n).map_err(|_| AntiFraudeError::Repository("overflow".into()))
    }

    async fn disputes_as_counterparty(&self, user_id: Uuid) -> Result<u32, AntiFraudeError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*)::BIGINT AS n
            FROM disputes d
            JOIN orders o ON d.order_id = o.id
            WHERE (o.seller_id = $1 OR o.buyer_id = $1)
              AND d.opened_by_user_id != $1
              AND d.status IN ('open', 'under_review')
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        let n: i64 = row.try_get("n").map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        u32::try_from(n).map_err(|_| AntiFraudeError::Repository("overflow".into()))
    }

    async fn transaction_count(&self, user_id: Uuid, window_hours: u32) -> Result<u32, AntiFraudeError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*)::BIGINT AS n FROM orders
            WHERE buyer_id = $1
              AND created_at > NOW() - ($2::BIGINT * INTERVAL '1 hour')
              AND status NOT IN ('cancelled', 'failed')
            "#,
        )
        .bind(user_id)
        .bind(window_hours as i64)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        let n: i64 = row.try_get("n").map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        u32::try_from(n).map_err(|_| AntiFraudeError::Repository("overflow".into()))
    }

    async fn transaction_volume(&self, user_id: Uuid, window_hours: u32) -> Result<Decimal, AntiFraudeError> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(amount), 0)::NUMERIC AS total FROM orders
            WHERE buyer_id = $1
              AND created_at > NOW() - ($2::BIGINT * INTERVAL '1 hour')
              AND status NOT IN ('cancelled', 'failed')
            "#,
        )
        .bind(user_id)
        .bind(window_hours as i64)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        let total: rust_decimal::Decimal = row
            .try_get("total")
            .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        Ok(total)
    }

    async fn total_transaction_count(&self, user_id: Uuid) -> Result<u32, AntiFraudeError> {
        let row = sqlx::query(
            "SELECT COUNT(*)::BIGINT AS n FROM orders WHERE buyer_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        let n: i64 = row.try_get("n").map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        u32::try_from(n).map_err(|_| AntiFraudeError::Repository("overflow".into()))
    }

    async fn average_transaction_value(&self, user_id: Uuid) -> Result<Option<Decimal>, AntiFraudeError> {
        let row = sqlx::query(
            "SELECT AVG(amount)::NUMERIC AS avg FROM orders
             WHERE buyer_id = $1 AND status NOT IN ('cancelled', 'failed')",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        let avg: Option<Decimal> = row
            .try_get("avg")
            .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        Ok(avg)
    }

    async fn account_age_days(&self, user_id: Uuid) -> Result<u32, AntiFraudeError> {
        // Proxy: days since first order (no separate users table required).
        let row = sqlx::query(
            r#"
            SELECT EXTRACT(DAY FROM NOW() - MIN(created_at))::BIGINT AS age
            FROM orders WHERE buyer_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        let age: Option<i64> = row
            .try_get("age")
            .map_err(|e| AntiFraudeError::Repository(e.to_string()))?;
        Ok(age.map(|d| d.max(0) as u32).unwrap_or(0))
    }
}

// ─── helpers ─────────────────────────────────────────────────────────────────

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
        _ => Err(AntiFraudeError::Repository(format!("unknown risk level: {s}"))),
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
        _ => Err(AntiFraudeError::Repository(format!("unknown decision: {s}"))),
    }
}
