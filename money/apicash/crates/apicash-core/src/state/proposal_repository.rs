//! Persistence boundary for proposals (two-party escrow negotiation).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::dto::{ProposalStatus, StoredProposal};

#[async_trait]
pub trait ProposalRepository: Send + Sync {
    async fn save(&self, proposal: StoredProposal) -> Result<(), String>;
    async fn get(&self, id: Uuid) -> Result<Option<StoredProposal>, String>;
    async fn update(&self, proposal: StoredProposal) -> Result<(), String>;
    async fn list_by_seller(&self, seller_id: Uuid) -> Result<Vec<StoredProposal>, String>;
}

// ---------------------------------------------------------------------------
// In-memory (tests / no-DB mode)
// ---------------------------------------------------------------------------

pub struct InMemoryProposalRepository {
    by_id: Arc<RwLock<HashMap<Uuid, StoredProposal>>>,
}

impl InMemoryProposalRepository {
    pub fn new() -> Self {
        Self {
            by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryProposalRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProposalRepository for InMemoryProposalRepository {
    async fn save(&self, proposal: StoredProposal) -> Result<(), String> {
        self.by_id.write().await.insert(proposal.id, proposal);
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<StoredProposal>, String> {
        Ok(self.by_id.read().await.get(&id).cloned())
    }

    async fn update(&self, proposal: StoredProposal) -> Result<(), String> {
        let mut g = self.by_id.write().await;
        if !g.contains_key(&proposal.id) {
            return Err(format!("proposal not found: {}", proposal.id));
        }
        g.insert(proposal.id, proposal);
        Ok(())
    }

    async fn list_by_seller(&self, seller_id: Uuid) -> Result<Vec<StoredProposal>, String> {
        Ok(self
            .by_id
            .read()
            .await
            .values()
            .filter(|p| p.seller_id == seller_id)
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// Postgres
// ---------------------------------------------------------------------------

pub struct PostgresProposalRepository {
    pool: PgPool,
}

impl PostgresProposalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProposalRepository for PostgresProposalRepository {
    async fn save(&self, p: StoredProposal) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO proposals (id, seller_id, buyer_id, amount, description, status,
                                   created_at, expires_at, order_id, listing_id, seller_document)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(p.id)
        .bind(p.seller_id)
        .bind(p.buyer_id)
        .bind(&p.amount)
        .bind(p.description.as_deref())
        .bind(p.status.to_string())
        .bind(p.created_at)
        .bind(p.expires_at)
        .bind(p.order_id)
        .bind(p.listing_id)
        .bind(p.seller_document.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<StoredProposal>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, seller_id, buyer_id, amount, description, status,
                   created_at, expires_at, order_id, listing_id, seller_document
            FROM proposals WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        row.map(|r| row_to_stored(&r)).transpose()
    }

    async fn update(&self, p: StoredProposal) -> Result<(), String> {
        let n = sqlx::query(
            r#"
            UPDATE proposals SET
                status      = $2,
                order_id    = $3,
                buyer_id    = $4,
                description = $5
            WHERE id = $1
            "#,
        )
        .bind(p.id)
        .bind(p.status.to_string())
        .bind(p.order_id)
        .bind(p.buyer_id)
        .bind(p.description.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?
        .rows_affected();

        if n == 0 {
            return Err(format!("proposal not found: {}", p.id));
        }
        Ok(())
    }

    async fn list_by_seller(&self, seller_id: Uuid) -> Result<Vec<StoredProposal>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, seller_id, buyer_id, amount, description, status,
                   created_at, expires_at, order_id, listing_id, seller_document
            FROM proposals WHERE seller_id = $1 ORDER BY created_at DESC
            "#,
        )
        .bind(seller_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        rows.iter().map(row_to_stored).collect()
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn row_to_stored(r: &sqlx::postgres::PgRow) -> Result<StoredProposal, String> {
    let status_str: String = r.try_get("status").map_err(|e| e.to_string())?;
    let status = proposal_status_from_str(&status_str)?;
    let created_at: DateTime<Utc> = r.try_get("created_at").map_err(|e| e.to_string())?;
    let expires_at: DateTime<Utc> = r.try_get("expires_at").map_err(|e| e.to_string())?;

    Ok(StoredProposal {
        id: r.try_get("id").map_err(|e| e.to_string())?,
        seller_id: r.try_get("seller_id").map_err(|e| e.to_string())?,
        seller_document: r.try_get("seller_document").map_err(|e| e.to_string())?,
        buyer_id: r.try_get("buyer_id").map_err(|e| e.to_string())?,
        amount: r.try_get("amount").map_err(|e| e.to_string())?,
        description: r.try_get("description").map_err(|e| e.to_string())?,
        status,
        created_at,
        expires_at,
        order_id: r.try_get("order_id").map_err(|e| e.to_string())?,
        listing_id: r.try_get("listing_id").map_err(|e| e.to_string())?,
    })
}

fn proposal_status_from_str(s: &str) -> Result<ProposalStatus, String> {
    match s {
        "pending"  => Ok(ProposalStatus::Pending),
        "accepted" => Ok(ProposalStatus::Accepted),
        "rejected" => Ok(ProposalStatus::Rejected),
        "expired"  => Ok(ProposalStatus::Expired),
        _ => Err(format!("unknown proposal status: {s}")),
    }
}
