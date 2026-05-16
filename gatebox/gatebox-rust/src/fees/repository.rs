use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use crate::model::Fees;
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
pub trait FeesRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Fees>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Fees>, RepositoryError>;
    async fn get_by_account_id(&self, account_id: i64) -> Result<Option<Fees>, RepositoryError>;
    async fn insert(&self, item: &Fees) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &Fees) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct FeesRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl FeesRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    account_id: i64,
    fixed_cash_in: rust_decimal::Decimal,
    fixed_cash_out: rust_decimal::Decimal,
    percent_cashin: rust_decimal::Decimal,
    percent_cashout: rust_decimal::Decimal,
    percentsec_med: rust_decimal::Decimal,
    fixed_ref_cash_in: rust_decimal::Decimal,
    fixed_ref_cash_out: rust_decimal::Decimal,
    apagar: String,
    percent_ref_cashin: rust_decimal::Decimal,
    percent_ref_cashout: rust_decimal::Decimal,
    deleted_at: Option<DateTime<Utc>>,
}

fn to_fees(r: Row) -> Fees {
    Fees {
        id: r.id,
        account_id: r.account_id,
        fixed_cash_in: r.fixed_cash_in,
        fixed_cash_out: r.fixed_cash_out,
        percent_cashin: r.percent_cashin,
        percent_cashout: r.percent_cashout,
        percentsec_med: r.percentsec_med,
        fixed_ref_cash_in: r.fixed_ref_cash_in,
        fixed_ref_cash_out: r.fixed_ref_cash_out,
        apagar: r.apagar,
        percent_ref_cashin: r.percent_ref_cashin,
        percent_ref_cashout: r.percent_ref_cashout,
        deleted_at: r.deleted_at,
        full_count: None,
    }
}

#[async_trait]
impl FeesRepository for FeesRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Fees>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST).bind(limit).bind(offset).fetch_all(self.read.as_ref()).await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_fees).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Fees>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID).bind(id).fetch_optional(self.read.as_ref()).await?;
        Ok(row.map(to_fees))
    }
    async fn get_by_account_id(&self, account_id: i64) -> Result<Option<Fees>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ACCOUNT_ID)
            .bind(account_id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_fees))
    }
    async fn insert(&self, item: &Fees) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(item.account_id)
            .bind(item.fixed_cash_in)
            .bind(item.fixed_cash_out)
            .bind(item.percent_cashin)
            .bind(item.percent_cashout)
            .bind(item.percentsec_med)
            .bind(item.fixed_ref_cash_in)
            .bind(item.fixed_ref_cash_out)
            .bind(&item.apagar)
            .bind(item.percent_ref_cashin)
            .bind(item.percent_ref_cashout)
            .bind(item.deleted_at)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &Fees) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(item.account_id)
            .bind(item.fixed_cash_in)
            .bind(item.fixed_cash_out)
            .bind(item.percent_cashin)
            .bind(item.percent_cashout)
            .bind(item.percentsec_med)
            .bind(item.fixed_ref_cash_in)
            .bind(item.fixed_ref_cash_out)
            .bind(&item.apagar)
            .bind(item.percent_ref_cashin)
            .bind(item.percent_ref_cashout)
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
