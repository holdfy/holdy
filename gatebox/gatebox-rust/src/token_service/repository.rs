use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::TokenService;
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
pub trait TokenServiceRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<TokenService>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<TokenService>, RepositoryError>;
    async fn insert(&self, item: &TokenService) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &TokenService) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct TokenServiceRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl TokenServiceRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    description: String,
    token: String,
    expire_in: String,
    authentication_id: i64,
    timestamp: Option<DateTime<Utc>>,
    active: bool,
}

fn to_token_service(r: Row) -> TokenService {
    TokenService {
        id: r.id,
        description: r.description,
        token: r.token,
        expire_in: r.expire_in,
        authentication_id: r.authentication_id,
        timestamp: r.timestamp,
        active: r.active,
        full_count: None,
    }
}

#[async_trait]
impl TokenServiceRepository for TokenServiceRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<TokenService>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_token_service).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<TokenService>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_token_service))
    }
    async fn insert(&self, item: &TokenService) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.description)
            .bind(&item.token)
            .bind(&item.expire_in)
            .bind(item.authentication_id)
            .bind(item.timestamp)
            .bind(item.active)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &TokenService) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.description)
            .bind(&item.token)
            .bind(&item.expire_in)
            .bind(item.authentication_id)
            .bind(item.timestamp)
            .bind(item.active)
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
