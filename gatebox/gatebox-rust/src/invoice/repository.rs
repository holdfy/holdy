use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::Invoice;
use crate::shared::types::ItemsPage;
use super::ddl;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not found")]
    NotFound,
}

/// Result of get_by_external_id: (id, invoice_type_id, partners_list_id, gateway_description).
pub type InvoiceByExternalId = (i64, i64, i64, Option<String>);

#[async_trait]
pub trait InvoiceRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Invoice>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Invoice>, RepositoryError>;
    async fn get_by_external_id(&self, external_id: &str) -> Result<Option<InvoiceByExternalId>, RepositoryError>;
    async fn update_status(&self, id: i64, invoice_status_id: i64) -> Result<(), RepositoryError>;
    async fn insert(&self, item: &Invoice) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &Invoice) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct InvoiceRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl InvoiceRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    identifier: String,
    key: String,
    pix_key_type_id: i64,
    invoice_type_id: i64,
    timeout: i64,
    expire: i64,
    partners_list_id: i64,
    amount: Decimal,
    invoice_status_id: i64,
    external_id: String,
    document_number: String,
    description: String,
    account_id: i64,
    deleted_at: Option<DateTime<Utc>>,
}

fn to_invoice(r: Row) -> Invoice {
    Invoice {
        id: r.id,
        identifier: r.identifier,
        key: r.key,
        pix_key_type_id: r.pix_key_type_id,
        invoice_type_id: r.invoice_type_id,
        timeout: r.timeout,
        expire: r.expire,
        partners_list_id: r.partners_list_id,
        amount: r.amount,
        invoice_status_id: r.invoice_status_id,
        external_id: r.external_id,
        document_number: r.document_number,
        description: r.description,
        account_id: r.account_id,
        deleted_at: r.deleted_at,
        full_count: None,
    }
}

#[async_trait]
impl InvoiceRepository for InvoiceRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Invoice>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_invoice).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Invoice>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_invoice))
    }
    async fn get_by_external_id(&self, external_id: &str) -> Result<Option<InvoiceByExternalId>, RepositoryError> {
        #[derive(sqlx::FromRow)]
        struct InvRow {
            id: i64,
            invoice_type_id: i64,
            partners_list_id: i64,
            description: Option<String>,
        }
        let row: Option<InvRow> = sqlx::query_as(ddl::SQL_GET_BY_EXTERNAL_ID)
            .bind(external_id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(|r| (r.id, r.invoice_type_id, r.partners_list_id, r.description)))
    }
    async fn update_status(&self, id: i64, invoice_status_id: i64) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE_STATUS)
            .bind(invoice_status_id)
            .bind(id)
            .execute(self.write.as_ref())
            .await?;
        if r.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
    async fn insert(&self, item: &Invoice) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.identifier)
            .bind(&item.key)
            .bind(item.pix_key_type_id)
            .bind(item.invoice_type_id)
            .bind(item.timeout)
            .bind(item.expire)
            .bind(item.partners_list_id)
            .bind(item.amount)
            .bind(item.invoice_status_id)
            .bind(&item.external_id)
            .bind(&item.document_number)
            .bind(&item.description)
            .bind(item.account_id)
            .bind(item.deleted_at)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &Invoice) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.identifier)
            .bind(&item.key)
            .bind(item.pix_key_type_id)
            .bind(item.invoice_type_id)
            .bind(item.timeout)
            .bind(item.expire)
            .bind(item.partners_list_id)
            .bind(item.amount)
            .bind(item.invoice_status_id)
            .bind(&item.external_id)
            .bind(&item.document_number)
            .bind(&item.description)
            .bind(item.account_id)
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
