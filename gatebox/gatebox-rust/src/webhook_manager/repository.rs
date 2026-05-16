use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::WebhookManager;
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
pub trait WebhookManagerRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<WebhookManager>>, RepositoryError>;
    async fn list_by_account(&self, account_id: i64, offset: i64, limit: i64) -> Result<Vec<WebhookManager>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<WebhookManager>, RepositoryError>;
    async fn insert(&self, item: &WebhookManager) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &WebhookManager) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct WebhookManagerRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl WebhookManagerRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    callback_url: String,
    username: String,
    password: String,
    api_key: String,
    webhook_type_id: i64,
    account_id: i64,
    deleted_at: Option<DateTime<Utc>>,
}

fn to_webhook_manager(r: Row) -> WebhookManager {
    WebhookManager {
        id: r.id,
        callback_url: r.callback_url,
        username: r.username,
        password: r.password,
        api_key: r.api_key,
        webhook_type_id: r.webhook_type_id,
        account_id: r.account_id,
        deleted_at: r.deleted_at,
        full_count: None,
    }
}

#[async_trait]
impl WebhookManagerRepository for WebhookManagerRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<WebhookManager>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_webhook_manager).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn list_by_account(&self, account_id: i64, offset: i64, limit: i64) -> Result<Vec<WebhookManager>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST_BY_ACCOUNT)
            .bind(account_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        Ok(rows.into_iter().map(to_webhook_manager).collect())
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<WebhookManager>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_webhook_manager))
    }
    async fn insert(&self, item: &WebhookManager) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.callback_url)
            .bind(&item.username)
            .bind(&item.password)
            .bind(&item.api_key)
            .bind(item.webhook_type_id)
            .bind(item.account_id)
            .bind(item.deleted_at)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &WebhookManager) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.callback_url)
            .bind(&item.username)
            .bind(&item.password)
            .bind(&item.api_key)
            .bind(item.webhook_type_id)
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
