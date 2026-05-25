// Accounts repository - converted from gateboxgo/app/accounts/repository/accounts.go
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::Accounts;
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
pub trait AccountsRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Accounts>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Accounts>, RepositoryError>;
    async fn get_by_authentication_id(&self, authentication_id: i64) -> Result<Option<Accounts>, RepositoryError>;
    async fn insert(&self, item: &Accounts) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &Accounts) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct AccountsRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl AccountsRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[async_trait]
impl AccountsRepository for AccountsRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Accounts>>, RepositoryError> {
        let rows: Vec<AccountsRow> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(Accounts::from).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Accounts>, RepositoryError> {
        let row = sqlx::query_as::<_, AccountsRow>(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(Accounts::from))
    }

    async fn get_by_authentication_id(&self, authentication_id: i64) -> Result<Option<Accounts>, RepositoryError> {
        let row = sqlx::query_as::<_, AccountsRow>(ddl::SQL_GET_BY_AUTHENTICATION_ID)
            .bind(authentication_id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(Accounts::from))
    }

    async fn insert(&self, item: &Accounts) -> Result<i64, RepositoryError> {
        let (id,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.account_number)
            .bind(&item.branch)
            .bind(item.account_type_id)
            .bind(item.account_status_id)
            .bind(item.deleted_at)
            .bind(item.authentication_id)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }

    async fn update(&self, id: i64, item: &Accounts) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.account_number)
            .bind(&item.branch)
            .bind(item.account_type_id)
            .bind(item.account_status_id)
            .bind(item.deleted_at)
            .bind(item.authentication_id)
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

#[derive(sqlx::FromRow)]
struct AccountsRow {
    id: i64,
    account_number: String,
    branch: String,
    account_type_id: i64,
    account_status_id: i64,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    authentication_id: i64,
    type_person_id: Option<i64>,
}

impl From<AccountsRow> for Accounts {
    fn from(r: AccountsRow) -> Self {
        Accounts {
            id: r.id,
            account_number: r.account_number,
            branch: r.branch,
            account_type_id: r.account_type_id,
            account_status_id: r.account_status_id,
            deleted_at: r.deleted_at,
            authentication_id: r.authentication_id,
            type_person_id: r.type_person_id,
            full_count: None,
        }
    }
}
