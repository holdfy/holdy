use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::ControlMed;
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
pub trait ControlMedRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<ControlMed>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<ControlMed>, RepositoryError>;
    async fn insert(&self, item: &ControlMed) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &ControlMed) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct ControlMedRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl ControlMedRepositoryImpl {
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
    bank_id: String,
    endtoend: String,
    details: String,
    status_controle_med_id: i64,
    amount: Decimal,
    data_med: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

fn to_control_med(r: Row) -> ControlMed {
    ControlMed {
        id: r.id,
        account_id: r.account_id,
        invoice_id: r.invoice_id,
        partners_id: r.partners_id,
        bank_id: r.bank_id,
        endtoend: r.endtoend,
        details: r.details,
        status_controle_med_id: r.status_controle_med_id,
        amount: r.amount,
        data_med: r.data_med,
        deleted_at: r.deleted_at,
        full_count: None,
    }
}

#[async_trait]
impl ControlMedRepository for ControlMedRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<ControlMed>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_control_med).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<ControlMed>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_control_med))
    }
    async fn insert(&self, item: &ControlMed) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(item.account_id)
            .bind(item.invoice_id)
            .bind(item.partners_id)
            .bind(&item.bank_id)
            .bind(&item.endtoend)
            .bind(&item.details)
            .bind(item.status_controle_med_id)
            .bind(item.amount)
            .bind(item.data_med)
            .bind(item.deleted_at)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &ControlMed) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(item.account_id)
            .bind(item.invoice_id)
            .bind(item.partners_id)
            .bind(&item.bank_id)
            .bind(&item.endtoend)
            .bind(&item.details)
            .bind(item.status_controle_med_id)
            .bind(item.amount)
            .bind(item.data_med)
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
