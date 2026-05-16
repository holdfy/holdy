use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use super::ddl;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AppLogRow {
    pub id: i64,
    pub level: String,
    pub service: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database: {0}")]
    Database(#[from] sqlx::Error),
}

#[async_trait]
pub trait AppLogRepository: Send + Sync {
    async fn list(
        &self,
        level: Option<&str>,
        service: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<AppLogRow>, RepositoryError>;
}

pub struct AppLogRepositoryImpl {
    read: Arc<PgPool>,
}

impl AppLogRepositoryImpl {
    pub fn new(read: Arc<PgPool>) -> Self {
        Self { read }
    }
}

#[async_trait]
impl AppLogRepository for AppLogRepositoryImpl {
    async fn list(
        &self,
        level: Option<&str>,
        service: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<AppLogRow>, RepositoryError> {
        let level_val = level.unwrap_or("");
        let service_val = service.unwrap_or("");
        let rows: Vec<(i64, String, String, String, DateTime<Utc>)> = sqlx::query_as(ddl::SQL_LIST)
            .bind(level_val)
            .bind(service_val)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        Ok(rows
            .into_iter()
            .map(|(id, level, service, message, created_at)| AppLogRow {
                id,
                level,
                service,
                message,
                created_at,
            })
            .collect())
    }
}
