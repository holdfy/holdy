//! Persistência de evidências individuais (tabela `dispute_evidence`).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::DisputeError;
use crate::models::evidence::{EvidenceKind, EvidenceParty, EvidenceRow};

#[async_trait]
pub trait EvidenceRepository: Send + Sync {
    async fn insert(&self, row: EvidenceRow) -> Result<(), DisputeError>;
    async fn list_for_dispute(&self, dispute_id: Uuid) -> Result<Vec<EvidenceRow>, DisputeError>;
    async fn mark_flagged(&self, id: Uuid) -> Result<(), DisputeError>;
}

// ─── In-memory (testes) ───────────────────────────────────────────────────────

pub struct InMemoryEvidenceRepository {
    rows: Arc<RwLock<HashMap<Uuid, EvidenceRow>>>,
}

impl InMemoryEvidenceRepository {
    pub fn new() -> Self {
        Self { rows: Arc::new(RwLock::new(HashMap::new())) }
    }
    pub fn shared(self) -> Arc<Self> { Arc::new(self) }
}

impl Default for InMemoryEvidenceRepository {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl EvidenceRepository for InMemoryEvidenceRepository {
    async fn insert(&self, row: EvidenceRow) -> Result<(), DisputeError> {
        self.rows.write().await.insert(row.id, row);
        Ok(())
    }

    async fn list_for_dispute(&self, dispute_id: Uuid) -> Result<Vec<EvidenceRow>, DisputeError> {
        let g = self.rows.read().await;
        Ok(g.values().filter(|r| r.dispute_id == dispute_id).cloned().collect())
    }

    async fn mark_flagged(&self, id: Uuid) -> Result<(), DisputeError> {
        let mut g = self.rows.write().await;
        if let Some(r) = g.get_mut(&id) { r.ai_flagged = true; }
        Ok(())
    }
}

// ─── Postgres ────────────────────────────────────────────────────────────────

pub struct PostgresEvidenceRepository {
    pool: PgPool,
}

impl PostgresEvidenceRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl EvidenceRepository for PostgresEvidenceRepository {
    async fn insert(&self, row: EvidenceRow) -> Result<(), DisputeError> {
        sqlx::query(
            r#"
            INSERT INTO dispute_evidence
                (id, dispute_id, uploaded_by, party, kind, minio_key, content, sha256, ai_flagged, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(row.id)
        .bind(row.dispute_id)
        .bind(row.uploaded_by)
        .bind(row.party.to_str())
        .bind(row.kind.to_str())
        .bind(row.minio_key.as_deref())
        .bind(row.content.as_deref())
        .bind(&row.sha256)
        .bind(row.ai_flagged)
        .bind(row.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DisputeError::Repository(e.to_string()))?;
        Ok(())
    }

    async fn list_for_dispute(&self, dispute_id: Uuid) -> Result<Vec<EvidenceRow>, DisputeError> {
        let rows = sqlx::query(
            r#"
            SELECT id, dispute_id, uploaded_by, party, kind,
                   minio_key, content, sha256, ai_flagged, created_at
            FROM dispute_evidence
            WHERE dispute_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(dispute_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DisputeError::Repository(e.to_string()))?;

        let endpoint = std::env::var("MINIO_ENDPOINT").unwrap_or_default();
        let bucket   = std::env::var("MINIO_DISPUTES_BUCKET")
            .unwrap_or_else(|_| "holdfy-disputes".to_string());

        rows.iter().map(|r| {
            let minio_key: Option<String> = r.try_get("minio_key").unwrap_or(None);
            let minio_url = minio_key.as_deref()
                .map(|k| format!("{endpoint}/{bucket}/{k}"));
            Ok(EvidenceRow {
                id:          r.try_get("id").map_err(|e| DisputeError::Repository(e.to_string()))?,
                dispute_id:  r.try_get("dispute_id").map_err(|e| DisputeError::Repository(e.to_string()))?,
                uploaded_by: r.try_get("uploaded_by").map_err(|e| DisputeError::Repository(e.to_string()))?,
                party:       EvidenceParty::from_str(r.try_get("party").unwrap_or("buyer")),
                kind:        EvidenceKind::from_str(r.try_get("kind").unwrap_or("other")),
                minio_key,
                minio_url,
                content:     r.try_get("content").unwrap_or(None),
                sha256:      r.try_get("sha256").map_err(|e| DisputeError::Repository(e.to_string()))?,
                ai_flagged:  r.try_get("ai_flagged").unwrap_or(false),
                created_at:  r.try_get("created_at").unwrap_or_else(|_| Utc::now()),
            })
        }).collect()
    }

    async fn mark_flagged(&self, id: Uuid) -> Result<(), DisputeError> {
        sqlx::query("UPDATE dispute_evidence SET ai_flagged = TRUE WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DisputeError::Repository(e.to_string()))?;
        Ok(())
    }
}
