use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::SecMed;
use crate::shared::types::ItemsPage;
use super::ddl;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not found")]
    NotFound,
}

#[async_trait]
pub trait SecMedRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<SecMed>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<SecMed>, RepositoryError>;
    async fn insert(&self, item: &SecMed) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &SecMed) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct SecMedRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl SecMedRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    account_id: i64,
    invoice_id: i64,
    partners_id: i64,
    apagar: String,
    transaction_id: i64,
    status_sec_med_id: i64,
    amount: Decimal,
    scheduled_date: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

fn to_sec_med(r: Row) -> SecMed {
    SecMed {
        id: r.id,
        account_id: r.account_id,
        invoice_id: r.invoice_id,
        partners_id: r.partners_id,
        apagar: r.apagar,
        transaction_id: r.transaction_id,
        status_sec_med_id: r.status_sec_med_id,
        amount: r.amount,
        scheduled_date: r.scheduled_date,
        deleted_at: r.deleted_at,
        full_count: None,
    }
}

#[async_trait]
impl SecMedRepository for SecMedRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<SecMed>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_sec_med).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<SecMed>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_sec_med))
    }
    async fn insert(&self, item: &SecMed) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(item.account_id)
            .bind(item.invoice_id)
            .bind(item.partners_id)
            .bind(&item.apagar)
            .bind(item.transaction_id)
            .bind(item.status_sec_med_id)
            .bind(item.amount)
            .bind(item.scheduled_date)
            .bind(item.deleted_at)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &SecMed) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(item.account_id)
            .bind(item.invoice_id)
            .bind(item.partners_id)
            .bind(&item.apagar)
            .bind(item.transaction_id)
            .bind(item.status_sec_med_id)
            .bind(item.amount)
            .bind(item.scheduled_date)
            .bind(item.deleted_at)
            .bind(id)
            .execute(self.write.as_ref())
            .await?;
        if r.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError> {
        let r = sqlx::query(ddl::SQL_DELETE).bind(id).execute(self.write.as_ref()).await?;
        Ok(r.rows_affected() > 0)
    }
}
