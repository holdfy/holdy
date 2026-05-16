//! Persistência de disputas (memória para testes; Postgres via SQLx em produção).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::DisputeError;
use crate::models::{Dispute, DisputeParty, DisputeStatus, Evidence, ResolutionType};

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
        resolved_at: Option<DateTime<Utc>>,
        resolution_type: Option<String>,
        resolution_notes: Option<String>,
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
            resolved_at,
            resolution_type: resolution_type
                .map(|s| resolution_from_str(&s))
                .transpose()?,
            resolution_notes,
        })
    }
}

#[async_trait]
impl DisputeRepository for PostgresDisputeRepository {
    async fn insert(&self, dispute: Dispute) -> Result<(), DisputeError> {
        let evidence = serde_json::to_value(&dispute.evidence)
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let resolution = dispute
            .resolution_type
            .map(resolution_to_str)
            .map(String::from);

        sqlx::query(
            r#"
            INSERT INTO disputes (
                id, order_id, opened_by, opened_by_user_id, reason, status, evidence,
                opened_at, resolved_at, resolution_type, resolution_notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
        .bind(dispute.resolved_at)
        .bind(resolution.as_deref())
        .bind(dispute.resolution_notes.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| DisputeError::Repository(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<Dispute>, DisputeError> {
        let row = sqlx::query(
            r#"
            SELECT id, order_id, opened_by, opened_by_user_id, reason, status, evidence,
                   opened_at, resolved_at, resolution_type, resolution_notes
            FROM disputes WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DisputeError::Repository(e.to_string()))?;

        let Some(r) = row else {
            return Ok(None);
        };

        let dispute_id: Uuid = r
            .try_get("id")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let order_id: Uuid = r
            .try_get("order_id")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let opened_by: String = r
            .try_get("opened_by")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let opened_by_user_id: Uuid = r
            .try_get("opened_by_user_id")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let reason: String = r
            .try_get("reason")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let status: String = r
            .try_get("status")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let evidence: serde_json::Value = r
            .try_get("evidence")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let opened_at: DateTime<Utc> = r
            .try_get("opened_at")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let resolved_at: Option<DateTime<Utc>> = r
            .try_get("resolved_at")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let resolution_type: Option<String> = r
            .try_get("resolution_type")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let resolution_notes: Option<String> = r
            .try_get("resolution_notes")
            .map_err(|e| DisputeError::Repository(e.to_string()))?;

        Ok(Some(Self::map_row(
            dispute_id,
            order_id,
            opened_by,
            opened_by_user_id,
            reason,
            status,
            evidence,
            opened_at,
            resolved_at,
            resolution_type,
            resolution_notes,
        )?))
    }

    async fn update(&self, dispute: Dispute) -> Result<(), DisputeError> {
        let evidence = serde_json::to_value(&dispute.evidence)
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        let resolution = dispute
            .resolution_type
            .map(resolution_to_str)
            .map(String::from);

        let n = sqlx::query(
            r#"
            UPDATE disputes SET
                order_id = $2,
                opened_by = $3,
                opened_by_user_id = $4,
                reason = $5,
                status = $6,
                evidence = $7,
                opened_at = $8,
                resolved_at = $9,
                resolution_type = $10,
                resolution_notes = $11
            WHERE id = $1
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
        .bind(dispute.resolved_at)
        .bind(resolution.as_deref())
        .bind(dispute.resolution_notes.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| DisputeError::Repository(e.to_string()))?
        .rows_affected();

        if n == 0 {
            return Err(DisputeError::NotFound(dispute.id));
        }
        Ok(())
    }

    async fn list_open(&self) -> Result<Vec<Dispute>, DisputeError> {
        let rows = sqlx::query(
            r#"
            SELECT id, order_id, opened_by, opened_by_user_id, reason, status, evidence,
                   opened_at, resolved_at, resolution_type, resolution_notes
            FROM disputes
            WHERE status IN ('open', 'under_review')
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DisputeError::Repository(e.to_string()))?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let id: Uuid = r
                .try_get("id")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let order_id: Uuid = r
                .try_get("order_id")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let opened_by: String = r
                .try_get("opened_by")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let opened_by_user_id: Uuid = r
                .try_get("opened_by_user_id")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let reason: String = r
                .try_get("reason")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let status: String = r
                .try_get("status")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let evidence: serde_json::Value = r
                .try_get("evidence")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let opened_at: DateTime<Utc> = r
                .try_get("opened_at")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let resolved_at: Option<DateTime<Utc>> = r
                .try_get("resolved_at")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let resolution_type: Option<String> = r
                .try_get("resolution_type")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let resolution_notes: Option<String> = r
                .try_get("resolution_notes")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;

            out.push(PostgresDisputeRepository::map_row(
                id,
                order_id,
                opened_by,
                opened_by_user_id,
                reason,
                status,
                evidence,
                opened_at,
                resolved_at,
                resolution_type,
                resolution_notes,
            )?);
        }
        Ok(out)
    }

    async fn list_all(&self) -> Result<Vec<Dispute>, DisputeError> {
        let rows = sqlx::query(
            r#"
            SELECT id, order_id, opened_by, opened_by_user_id, reason, status, evidence,
                   opened_at, resolved_at, resolution_type, resolution_notes
            FROM disputes
            ORDER BY opened_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DisputeError::Repository(e.to_string()))?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let id: Uuid = r
                .try_get("id")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let order_id: Uuid = r
                .try_get("order_id")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let opened_by: String = r
                .try_get("opened_by")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let opened_by_user_id: Uuid = r
                .try_get("opened_by_user_id")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let reason: String = r
                .try_get("reason")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let status: String = r
                .try_get("status")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let evidence: serde_json::Value = r
                .try_get("evidence")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let opened_at: DateTime<Utc> = r
                .try_get("opened_at")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let resolved_at: Option<DateTime<Utc>> = r
                .try_get("resolved_at")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let resolution_type: Option<String> = r
                .try_get("resolution_type")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;
            let resolution_notes: Option<String> = r
                .try_get("resolution_notes")
                .map_err(|e| DisputeError::Repository(e.to_string()))?;

            out.push(PostgresDisputeRepository::map_row(
                id,
                order_id,
                opened_by,
                opened_by_user_id,
                reason,
                status,
                evidence,
                opened_at,
                resolved_at,
                resolution_type,
                resolution_notes,
            )?);
        }
        Ok(out)
    }
}
