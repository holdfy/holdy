use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::AccountRules;
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
pub trait AccountRulesRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<AccountRules>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<AccountRules>, RepositoryError>;
    async fn get_by_account_id(&self, account_id: i64) -> Result<Option<AccountRules>, RepositoryError>;
    async fn insert(&self, item: &AccountRules) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &AccountRules) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct AccountRulesRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl AccountRulesRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    account_id: i64,
    receive_external: bool,
    deposit_external: bool,
    descricao: String,
}

fn to_account_rules(r: Row) -> AccountRules {
    AccountRules {
        id: r.id,
        account_id: r.account_id,
        receive_external: r.receive_external,
        deposit_external: r.deposit_external,
        descricao: r.descricao,
        full_count: None,
    }
}

#[async_trait]
impl AccountRulesRepository for AccountRulesRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<AccountRules>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_account_rules).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<AccountRules>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_account_rules))
    }
    async fn get_by_account_id(&self, account_id: i64) -> Result<Option<AccountRules>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ACCOUNT_ID)
            .bind(account_id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_account_rules))
    }
    async fn insert(&self, item: &AccountRules) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(item.account_id)
            .bind(item.receive_external)
            .bind(item.deposit_external)
            .bind(&item.descricao)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &AccountRules) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(item.account_id)
            .bind(item.receive_external)
            .bind(item.deposit_external)
            .bind(&item.descricao)
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
