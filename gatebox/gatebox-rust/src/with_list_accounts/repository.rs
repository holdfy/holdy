use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::WithListAccounts;
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
pub trait WithListAccountsRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<WithListAccounts>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<WithListAccounts>, RepositoryError>;
    /// type_external_id = 1 = PIX_OUT
    async fn is_whitelisted_for_pix_out(&self, account_id: i64) -> Result<bool, RepositoryError>;
    /// type_external_id = 2 = PIX_IN
    async fn is_whitelisted_for_pix_in(&self, account_id: i64) -> Result<bool, RepositoryError>;
    async fn insert(&self, item: &WithListAccounts) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &WithListAccounts) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct WithListAccountsRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl WithListAccountsRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    type_external_id: i64,
    account_id: i64,
    document: String,
}

fn to_with_list_accounts(r: Row) -> WithListAccounts {
    WithListAccounts {
        id: r.id,
        type_external_id: r.type_external_id,
        account_id: r.account_id,
        document: r.document,
        full_count: None,
    }
}

#[async_trait]
impl WithListAccountsRepository for WithListAccountsRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<WithListAccounts>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_with_list_accounts).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<WithListAccounts>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_with_list_accounts))
    }
    async fn is_whitelisted_for_pix_out(&self, account_id: i64) -> Result<bool, RepositoryError> {
        let row: (bool,) = sqlx::query_as(ddl::SQL_IS_WHITELISTED_PIX_OUT)
            .bind(account_id)
            .fetch_one(self.read.as_ref())
            .await?;
        Ok(row.0)
    }
    async fn is_whitelisted_for_pix_in(&self, account_id: i64) -> Result<bool, RepositoryError> {
        let row: (bool,) = sqlx::query_as(ddl::SQL_IS_WHITELISTED_PIX_IN)
            .bind(account_id)
            .fetch_one(self.read.as_ref())
            .await?;
        Ok(row.0)
    }
    async fn insert(&self, item: &WithListAccounts) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(item.type_external_id)
            .bind(item.account_id)
            .bind(&item.document)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &WithListAccounts) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(item.type_external_id)
            .bind(item.account_id)
            .bind(&item.document)
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
