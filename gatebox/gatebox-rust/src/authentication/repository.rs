use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use crate::model::Authentication;
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
pub trait AuthenticationRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Authentication>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Authentication>, RepositoryError>;
    async fn get_by_username(&self, username: &str) -> Result<Option<Authentication>, RepositoryError>;
    async fn get_by_username_and_type(&self, username: &str, type_auth_id: i64) -> Result<Option<Authentication>, RepositoryError>;
    async fn update_password(&self, id: i64, password: &str) -> Result<(), RepositoryError>;
    async fn insert(&self, item: &Authentication) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &Authentication) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct AuthenticationRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl AuthenticationRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    name: String,
    username: String,
    password: String,
    type_auth_id: i64,
    active: bool,
    force_reset: bool,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

fn to_authentication(r: Row) -> Authentication {
    Authentication {
        id: r.id,
        name: r.name,
        username: r.username,
        password: r.password,
        type_auth_id: r.type_auth_id,
        active: r.active,
        force_reset: r.force_reset,
        created_at: r.created_at,
        updated_at: r.updated_at,
        deleted_at: r.deleted_at,
        full_count: None,
    }
}

#[async_trait]
impl AuthenticationRepository for AuthenticationRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Authentication>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST).bind(limit).bind(offset).fetch_all(self.read.as_ref()).await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_authentication).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Authentication>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID).bind(id).fetch_optional(self.read.as_ref()).await?;
        Ok(row.map(to_authentication))
    }
    async fn get_by_username(&self, username: &str) -> Result<Option<Authentication>, RepositoryError> {
        let row: Option<Row> =
            sqlx::query_as(ddl::SQL_GET_BY_USERNAME).bind(username).fetch_optional(self.read.as_ref()).await?;
        Ok(row.map(to_authentication))
    }
    async fn get_by_username_and_type(&self, username: &str, type_auth_id: i64) -> Result<Option<Authentication>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_USERNAME_AND_TYPE)
            .bind(username)
            .bind(type_auth_id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_authentication))
    }
    async fn update_password(&self, id: i64, password: &str) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE_PASSWORD)
            .bind(password)
            .bind(id)
            .execute(self.write.as_ref())
            .await?;
        if r.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
    async fn insert(&self, item: &Authentication) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.name)
            .bind(&item.username)
            .bind(&item.password)
            .bind(item.type_auth_id)
            .bind(item.active)
            .bind(item.force_reset)
            .bind(item.created_at)
            .bind(item.updated_at)
            .bind(item.deleted_at)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &Authentication) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.name)
            .bind(&item.username)
            .bind(&item.password)
            .bind(item.type_auth_id)
            .bind(item.active)
            .bind(item.force_reset)
            .bind(item.created_at)
            .bind(item.updated_at)
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
