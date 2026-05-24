use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use super::ddl;
use super::model::Dispute;
use crate::shared::types::ItemsPage;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database: {0}")]
    Database(#[from] sqlx::Error),
}

#[async_trait]
pub trait DisputeRepository: Send + Sync {
    async fn insert(
        &self,
        transaction_id: Option<i64>,
        account_id: i64,
        dispute_type: &str,
        reason: &str,
        evidence: Option<&serde_json::Value>,
    ) -> Result<i64, RepositoryError>;

    async fn list(
        &self,
        status: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<Dispute>>, RepositoryError>;

    async fn list_by_account(
        &self,
        account_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<Dispute>>, RepositoryError>;

    async fn get_by_id(&self, id: i64) -> Result<Option<Dispute>, RepositoryError>;

    async fn resolve(
        &self,
        id: i64,
        resolved_by: Option<i64>,
        notes: Option<&str>,
    ) -> Result<bool, RepositoryError>;
}

pub struct DisputeRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl DisputeRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    transaction_id: Option<i64>,
    account_id: i64,
    r#type: String,
    status: String,
    reason: String,
    evidence: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
    resolved_by: Option<i64>,
    resolution_notes: Option<String>,
    full_count: Option<i64>,
}

fn to_dispute(r: Row) -> Dispute {
    Dispute {
        id: r.id,
        transaction_id: r.transaction_id,
        account_id: r.account_id,
        r#type: r.r#type,
        status: r.status,
        reason: r.reason,
        evidence: r.evidence,
        created_at: r.created_at,
        resolved_at: r.resolved_at,
        resolved_by: r.resolved_by,
        resolution_notes: r.resolution_notes,
        full_count: r.full_count,
    }
}

#[async_trait]
impl DisputeRepository for DisputeRepositoryImpl {
    async fn insert(
        &self,
        transaction_id: Option<i64>,
        account_id: i64,
        dispute_type: &str,
        reason: &str,
        evidence: Option<&serde_json::Value>,
    ) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(transaction_id)
            .bind(account_id)
            .bind(dispute_type)
            .bind(reason)
            .bind(evidence)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }

    async fn list(
        &self,
        status: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<Dispute>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(status.unwrap_or(""))
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.first().and_then(|r| r.full_count).unwrap_or(0);
        let items = rows.into_iter().map(to_dispute).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }

    async fn list_by_account(
        &self,
        account_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<Dispute>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST_BY_ACCOUNT)
            .bind(account_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.first().and_then(|r| r.full_count).unwrap_or(0);
        let items = rows.into_iter().map(to_dispute).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Dispute>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_dispute))
    }

    async fn resolve(
        &self,
        id: i64,
        resolved_by: Option<i64>,
        notes: Option<&str>,
    ) -> Result<bool, RepositoryError> {
        let r = sqlx::query(ddl::SQL_RESOLVE)
            .bind(id)
            .bind(resolved_by)
            .bind(notes)
            .execute(self.write.as_ref())
            .await?;
        Ok(r.rows_affected() > 0)
    }
}
