use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::SharedKey;
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
pub trait SharedKeyRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<SharedKey>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<SharedKey>, RepositoryError>;
    async fn insert(&self, item: &SharedKey) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &SharedKey) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct SharedKeyRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl SharedKeyRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    key: String,
    pix_key_type_id: i64,
    description: String,
    deleted_at: Option<DateTime<Utc>>,
}

fn to_shared_key(r: Row) -> SharedKey {
    SharedKey {
        id: r.id,
        key: r.key,
        pix_key_type_id: r.pix_key_type_id,
        description: r.description,
        deleted_at: r.deleted_at,
        full_count: None,
    }
}

#[async_trait]
impl SharedKeyRepository for SharedKeyRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<SharedKey>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_shared_key).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<SharedKey>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_shared_key))
    }
    async fn insert(&self, item: &SharedKey) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.key)
            .bind(item.pix_key_type_id)
            .bind(&item.description)
            .bind(item.deleted_at)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &SharedKey) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.key)
            .bind(item.pix_key_type_id)
            .bind(&item.description)
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
