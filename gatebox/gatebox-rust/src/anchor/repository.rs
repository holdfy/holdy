use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use super::ddl;
use super::types::TransactionAnchorRow;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database: {0}")]
    Database(#[from] sqlx::Error),
}

#[async_trait]
pub trait AnchorRepository: Send + Sync {
    async fn list(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        entity_type: Option<&str>,
        period_type: Option<&str>,
        period_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TransactionAnchorRow>, i64), RepositoryError>;
}

pub struct AnchorRepositoryImpl {
    read: Arc<PgPool>,
}

impl AnchorRepositoryImpl {
    pub fn new(read: Arc<PgPool>) -> Self {
        Self { read }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    idempotency_key: String,
    entity_type: String,
    entity_id: String,
    payload_hash: String,
    period_type: Option<String>,
    period_id: Option<String>,
    tx_hash: Option<String>,
    block_number: Option<i64>,
    chain_id: Option<i64>,
    anchored_at: Option<DateTime<Utc>>,
    dry_run: bool,
    error_message: Option<String>,
    account_id: i64,
    created_at: DateTime<Utc>,
}

fn to_row(r: Row) -> TransactionAnchorRow {
    TransactionAnchorRow {
        id: r.id,
        idempotency_key: r.idempotency_key,
        entity_type: r.entity_type,
        entity_id: r.entity_id,
        payload_hash: r.payload_hash,
        period_type: r.period_type,
        period_id: r.period_id,
        tx_hash: r.tx_hash,
        block_number: r.block_number,
        chain_id: r.chain_id,
        anchored_at: r.anchored_at,
        dry_run: r.dry_run,
        error_message: r.error_message,
        account_id: r.account_id,
        created_at: r.created_at,
    }
}

#[async_trait]
impl AnchorRepository for AnchorRepositoryImpl {
    async fn list(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        entity_type: Option<&str>,
        period_type: Option<&str>,
        period_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TransactionAnchorRow>, i64), RepositoryError> {
        let total: (i64,) = sqlx::query_as(ddl::SQL_COUNT)
            .bind(from)
            .bind(to)
            .bind(entity_type)
            .bind(period_type)
            .bind(period_id)
            .fetch_one(self.read.as_ref())
            .await?;

        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(from)
            .bind(to)
            .bind(entity_type)
            .bind(period_type)
            .bind(period_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;

        let items = rows.into_iter().map(to_row).collect();
        Ok((items, total.0))
    }
}
