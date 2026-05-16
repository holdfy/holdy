use async_trait::async_trait;
use std::sync::Arc;

use crate::model::SharedKey;
use crate::shared::types::ItemsPage;
use super::repository::{RepositoryError, SharedKeyRepository};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait SharedKeyService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<SharedKey>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<SharedKey>, ServiceError>;
    async fn create(&self, item: &SharedKey) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &SharedKey) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct SharedKeyServiceImpl {
    repo: Arc<dyn SharedKeyRepository>,
}
impl SharedKeyServiceImpl {
    pub fn new(repo: Arc<dyn SharedKeyRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl SharedKeyService for SharedKeyServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<SharedKey>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<SharedKey>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &SharedKey) -> Result<i64, ServiceError> {
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &SharedKey) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
