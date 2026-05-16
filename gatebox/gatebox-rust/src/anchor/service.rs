use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use super::repository::{AnchorRepository, RepositoryError};
use super::types::TransactionAnchorRow;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
}

#[async_trait]
pub trait AnchorService: Send + Sync {
    async fn list_audit(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        entity_type: Option<String>,
        period_type: Option<String>,
        period_id: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TransactionAnchorRow>, i64), ServiceError>;
}

pub struct AnchorServiceImpl {
    repo: Arc<dyn AnchorRepository>,
}

impl AnchorServiceImpl {
    pub fn new(repo: Arc<dyn AnchorRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AnchorService for AnchorServiceImpl {
    async fn list_audit(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        entity_type: Option<String>,
        period_type: Option<String>,
        period_id: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TransactionAnchorRow>, i64), ServiceError> {
        let limit = limit.clamp(1, 200);
        let offset = offset.max(0);
        let et = entity_type.as_deref();
        let pt = period_type.as_deref();
        let pid = period_id.as_deref();
        self.repo
            .list(from, to, et, pt, pid, limit, offset)
            .await
            .map_err(ServiceError::Repository)
    }
}
