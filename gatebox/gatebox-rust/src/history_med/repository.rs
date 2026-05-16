use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::HistoryMed;
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
pub trait HistoryMedRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<HistoryMed>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<HistoryMed>, RepositoryError>;
    async fn insert(&self, item: &HistoryMed) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &HistoryMed) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct HistoryMedRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl HistoryMedRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    account_id: i64,
    control_med_id: i64,
    sec_med_id: i64,
    apagar: String,
    amount: Decimal,
    deleted_at: Option<DateTime<Utc>>,
}

fn to_history_med(r: Row) -> HistoryMed {
    HistoryMed {
        id: r.id,
        account_id: r.account_id,
        control_med_id: r.control_med_id,
        sec_med_id: r.sec_med_id,
        apagar: r.apagar,
        amount: r.amount,
        deleted_at: r.deleted_at,
        full_count: None,
    }
}

#[async_trait]
impl HistoryMedRepository for HistoryMedRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<HistoryMed>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_history_med).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<HistoryMed>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_history_med))
    }
    async fn insert(&self, item: &HistoryMed) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(item.account_id)
            .bind(item.control_med_id)
            .bind(item.sec_med_id)
            .bind(&item.apagar)
            .bind(item.amount)
            .bind(item.deleted_at)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &HistoryMed) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(item.account_id)
            .bind(item.control_med_id)
            .bind(item.sec_med_id)
            .bind(&item.apagar)
            .bind(item.amount)
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
