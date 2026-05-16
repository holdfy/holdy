use async_trait::async_trait;
use std::sync::Arc;

use crate::model::KeyPixCache;
use crate::shared::types::ItemsPage;
use super::repository::{RepositoryError, KeyPixCacheRepository};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait KeyPixCacheService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<KeyPixCache>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<KeyPixCache>, ServiceError>;
    async fn create(&self, item: &KeyPixCache) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &KeyPixCache) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct KeyPixCacheServiceImpl {
    repo: Arc<dyn KeyPixCacheRepository>,
}
impl KeyPixCacheServiceImpl {
    pub fn new(repo: Arc<dyn KeyPixCacheRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl KeyPixCacheService for KeyPixCacheServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<KeyPixCache>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<KeyPixCache>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &KeyPixCache) -> Result<i64, ServiceError> {
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &KeyPixCache) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
