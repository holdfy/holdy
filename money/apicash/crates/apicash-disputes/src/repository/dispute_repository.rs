//! Persistência de disputas (memória para testes; Postgres via SQLx em produção).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::DisputeError;
use crate::models::{AiVerdict, Dispute, DisputeParty, DisputeStatus, Evidence, ResolutionType};

#[async_trait]
pub trait DisputeRepository: Send + Sync {
    async fn insert(&self, dispute: Dispute) -> Result<(), DisputeError>;
    async fn get(&self, id: Uuid) -> Result<Option<Dispute>, DisputeError>;
    async fn update(&self, dispute: Dispute) -> Result<(), DisputeError>;
    /// Disputas ainda não encerradas (`Open` ou `UnderReview`).
    async fn list_open(&self) -> Result<Vec<Dispute>, DisputeError>;
    /// Todas as disputas (painel admin).
    async fn list_all(&self) -> Result<Vec<Dispute>, DisputeError>;
}

fn party_from_str(s: &str) -> Result<DisputeParty, DisputeError> {
    match s {
        "buyer" => Ok(DisputeParty::Buyer),
        "seller" => Ok(DisputeParty::Seller),
        _ => Err(DisputeError::Validation(format!(
            "unknown dispute party: {s}"
        ))),
    }
}

fn status_from_str(s: &str) -> Result<DisputeStatus, DisputeError> {
    match s {
        "open" => Ok(DisputeStatus::Open),
        "under_review" => Ok(DisputeStatus::UnderReview),
        "resolved" => Ok(DisputeStatus::Resolved),
        "closed" => Ok(DisputeStatus::Closed),
        _ => Err(DisputeError::Validation(format!(
            "unknown dispute status: {s}"
        ))),
    }
}

fn resolution_from_str(s: &str) -> Result<ResolutionType, DisputeError> {
    match s {
        "refund_buyer" => Ok(ResolutionType::RefundBuyer),
        "release_to_seller" => Ok(ResolutionType::ReleaseToSeller),
        "split" => Ok(ResolutionType::Split),
        "manual" => Ok(ResolutionType::Manual),
        _ => Err(DisputeError::Validation(format!("unknown resolution: {s}"))),
    }
}

fn party_to_str(p: DisputeParty) -> &'static str {
    match p {
        DisputeParty::Buyer => "buyer",
        DisputeParty::Seller => "seller",
    }
}

fn status_to_str(s: DisputeStatus) -> &'static str {
    match s {
        DisputeStatus::Open => "open",
        DisputeStatus::UnderReview => "under_review",
        DisputeStatus::Resolved => "resolved",
        DisputeStatus::Closed => "closed",
    }
}

fn resolution_to_str(r: ResolutionType) -> &'static str {
    match r {
        ResolutionType::RefundBuyer => "refund_buyer",
        ResolutionType::ReleaseToSeller => "release_to_seller",
        ResolutionType::Split => "split",
        ResolutionType::Manual => "manual",
    }
}

/// Repositório em memória (testes e desenvolvimento).
pub struct InMemoryDisputeRepository {
    by_id: Arc<RwLock<HashMap<Uuid, Dispute>>>,
}

impl InMemoryDisputeRepository {
    pub fn new() -> Self {
        Self {
            by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

impl Default for InMemoryDisputeRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DisputeRepository for InMemoryDisputeRepository {
    async fn insert(&self, dispute: Dispute) -> Result<(), DisputeError> {
        let mut g = self.by_id.write().await;
        if g.contains_key(&dispute.id) {
            return Err(DisputeError::Validation("dispute id already exists".into()));
        }
        g.insert(dispute.id, dispute);
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<Dispute>, DisputeError> {
        Ok(self.by_id.read().await.get(&id).cloned())
    }

    async fn update(&self, dispute: Dispute) -> Result<(), DisputeError> {
        let mut g = self.by_id.write().await;
        if !g.contains_key(&dispute.id) {
            return Err(DisputeError::NotFound(dispute.id));
        }
        g.insert(dispute.id, dispute);
        Ok(())
    }

    async fn list_open(&self) -> Result<Vec<Dispute>, DisputeError> {
        let g = self.by_id.read().await;
        Ok(g.values()
            .filter(|d| matches!(d.status, DisputeStatus::Open | DisputeStatus::UnderReview))
            .cloned()
            .collect())
    }

    async fn list_all(&self) -> Result<Vec<Dispute>, DisputeError> {
        let g = self.by_id.read().await;
        Ok(g.values().cloned().collect())
    }
}

/// Repositório Postgres (SQLx runtime).
pub struct PostgresDisputeRepository {
    pool: PgPool,
}

impl PostgresDisputeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    fn map_row(
        id: Uuid,
        order_id: Uuid,
        opened_by: String,
        opened_by_user_id: Uuid,
        reason: String,
        status: String,
        evidence: serde_json::Value,
        opened_at: DateTime<Utc>,
        deadline_at: Option<DateTime<Utc>>,
        resolved_at: Option<DateTime<Utc>>,
        resolution_type: Option<String>,
        resolution_notes: Option<String>,
        ai_verdict: Option<String>,
        ai_confidence: Option<f32>,
        ai_reasoning: Option<String>,
        high_risk_buyer: bool,
    ) -> Result<Dispute, DisputeError> {
        let evidence: Vec<Evidence> = serde_json::from_value(evidence)
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        Ok(Dispute {
            id,
            order_id,
            opened_by: party_from_str(&opened_by)?,
            opened_by_user_id,
            reason,
            status: status_from_str(&status)?,
            evidence,
            opened_at,
            deadline_at,
            resolved_at,
            resolution_type: resolution_type
                .map(|s| resolution_from_str(&s))
                .transpose()?,
            resolution_notes,
            ai_verdict: ai_verdict.and_then(|s| AiVerdict::from_str(&s)),
            ai_confidence,
            ai_reasoning,
            high_risk_buyer,
        })
    }
}

#[async_trait]
impl DisputeRepository for PostgresDisputeRepository {
    async fn insert(&self, dispute: Dispute) -> Result<(), DisputeError> {
        let evidence = serde_json::to_value(&dispute.evidence)
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let resolution = dispute.resolution_type.map(resolution_to_str).map(String::from);
        let ai_verdict = dispute.ai_verdict.map(|v| v.to_str().to_string());

        sqlx::query(
            r#"
            INSERT INTO disputes (
                id, order_id, opened_by, opened_by_user_id, reason, status, evidence,
                opened_at, deadline_at, resolved_at, resolution_type, resolution_notes,
                ai_verdict, ai_confidence, ai_reasoning, high_risk_buyer
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            "#,
        )
        .bind(dispute.id)
        .bind(dispute.order_id)
        .bind(party_to_str(dispute.opened_by))
        .bind(dispute.opened_by_user_id)
        .bind(&dispute.reason)
        .bind(status_to_str(dispute.status))
        .bind(evidence)
        .bind(dispute.opened_at)
        .bind(dispute.deadline_at)
        .bind(dispute.resolved_at)
        .bind(resolution.as_deref())
        .bind(dispute.resolution_notes.as_deref())
        .bind(ai_verdict.as_deref())
        .bind(dispute.ai_confidence)
        .bind(dispute.ai_reasoning.as_deref())
        .bind(dispute.high_risk_buyer)
        .execute(&self.pool)
        .await
        .map_err(|e| DisputeError::Repository(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<Dispute>, DisputeError> {
        let row = sqlx::query(
            r#"
            SELECT id, order_id, opened_by, opened_by_user_id, reason, status, evidence,
                   opened_at, deadline_at, resolved_at, resolution_type, resolution_notes,
                   ai_verdict, ai_confidence, ai_reasoning,
                   COALESCE(high_risk_buyer, FALSE) AS high_risk_buyer
            FROM disputes WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DisputeError::Repository(e.to_string()))?;

        let Some(r) = row else { return Ok(None); };

        Ok(Some(Self::map_row(
            r.try_get("id").map_err(|e| DisputeError::Repository(e.to_string()))?,
            r.try_get("order_id").map_err(|e| DisputeError::Repository(e.to_string()))?,
            r.try_get("opened_by").map_err(|e| DisputeError::Repository(e.to_string()))?,
            r.try_get("opened_by_user_id").map_err(|e| DisputeError::Repository(e.to_string()))?,
            r.try_get("reason").map_err(|e| DisputeError::Repository(e.to_string()))?,
            r.try_get("status").map_err(|e| DisputeError::Repository(e.to_string()))?,
            r.try_get("evidence").map_err(|e| DisputeError::Repository(e.to_string()))?,
            r.try_get("opened_at").map_err(|e| DisputeError::Repository(e.to_string()))?,
            r.try_get("deadline_at").unwrap_or(None),
            r.try_get("resolved_at").unwrap_or(None),
            r.try_get("resolution_type").unwrap_or(None),
            r.try_get("resolution_notes").unwrap_or(None),
            r.try_get("ai_verdict").unwrap_or(None),
            r.try_get("ai_confidence").unwrap_or(None),
            r.try_get("ai_reasoning").unwrap_or(None),
            r.try_get("high_risk_buyer").unwrap_or(false),
        )?))
    }

    async fn update(&self, dispute: Dispute) -> Result<(), DisputeError> {
        let evidence   = serde_json::to_value(&dispute.evidence)
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let resolution = dispute.resolution_type.map(resolution_to_str).map(String::from);
        let ai_verdict = dispute.ai_verdict.map(|v| v.to_str().to_string());

        let n = sqlx::query(
            r#"
            UPDATE disputes SET
                status           = $2,
                evidence         = $3,
                deadline_at      = $4,
                resolved_at      = $5,
                resolution_type  = $6,
                resolution_notes = $7,
                ai_verdict       = $8,
                ai_confidence    = $9,
                ai_reasoning     = $10,
                high_risk_buyer  = $11
            WHERE id = $1
            "#,
        )
        .bind(dispute.id)
        .bind(status_to_str(dispute.status))
        .bind(evidence)
        .bind(dispute.deadline_at)
        .bind(dispute.resolved_at)
        .bind(resolution.as_deref())
        .bind(dispute.resolution_notes.as_deref())
        .bind(ai_verdict.as_deref())
        .bind(dispute.ai_confidence)
        .bind(dispute.ai_reasoning.as_deref())
        .bind(dispute.high_risk_buyer)
        .execute(&self.pool)
        .await
        .map_err(|e| DisputeError::Repository(e.to_string()))?
        .rows_affected();

        if n == 0 { return Err(DisputeError::NotFound(dispute.id)); }
        Ok(())
    }

    async fn list_open(&self) -> Result<Vec<Dispute>, DisputeError> {
        self.fetch_disputes(
            "SELECT id, order_id, opened_by, opened_by_user_id, reason, status, evidence, \
             opened_at, deadline_at, resolved_at, resolution_type, resolution_notes, \
             ai_verdict, ai_confidence, ai_reasoning, COALESCE(high_risk_buyer,FALSE) AS high_risk_buyer \
             FROM disputes WHERE status IN ('open','under_review') ORDER BY opened_at DESC",
        ).await
    }

    async fn list_all(&self) -> Result<Vec<Dispute>, DisputeError> {
        self.fetch_disputes(
            "SELECT id, order_id, opened_by, opened_by_user_id, reason, status, evidence, \
             opened_at, deadline_at, resolved_at, resolution_type, resolution_notes, \
             ai_verdict, ai_confidence, ai_reasoning, COALESCE(high_risk_buyer,FALSE) AS high_risk_buyer \
             FROM disputes ORDER BY opened_at DESC",
        ).await
    }
}

impl PostgresDisputeRepository {
    async fn fetch_disputes(&self, sql: &str) -> Result<Vec<Dispute>, DisputeError> {
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DisputeError::Repository(e.to_string()))?;

        rows.iter().map(|r| {
            Self::map_row(
                r.try_get("id").map_err(|e| DisputeError::Repository(e.to_string()))?,
                r.try_get("order_id").map_err(|e| DisputeError::Repository(e.to_string()))?,
                r.try_get("opened_by").map_err(|e| DisputeError::Repository(e.to_string()))?,
                r.try_get("opened_by_user_id").map_err(|e| DisputeError::Repository(e.to_string()))?,
                r.try_get("reason").map_err(|e| DisputeError::Repository(e.to_string()))?,
                r.try_get("status").map_err(|e| DisputeError::Repository(e.to_string()))?,
                r.try_get("evidence").map_err(|e| DisputeError::Repository(e.to_string()))?,
                r.try_get("opened_at").map_err(|e| DisputeError::Repository(e.to_string()))?,
                r.try_get("deadline_at").unwrap_or(None),
                r.try_get("resolved_at").unwrap_or(None),
                r.try_get("resolution_type").unwrap_or(None),
                r.try_get("resolution_notes").unwrap_or(None),
                r.try_get("ai_verdict").unwrap_or(None),
                r.try_get("ai_confidence").unwrap_or(None),
                r.try_get("ai_reasoning").unwrap_or(None),
                r.try_get("high_risk_buyer").unwrap_or(false),
            )
        }).collect()
    }
}
