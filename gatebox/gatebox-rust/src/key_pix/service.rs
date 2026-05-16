use async_trait::async_trait;
use std::sync::Arc;

use crate::model::KeyPix;
use crate::shared::types::ItemsPage;
use super::repository::{RepositoryError, KeyPixRepository};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait KeyPixService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<KeyPix>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<KeyPix>, ServiceError>;
    async fn create(&self, item: &KeyPix) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &KeyPix) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct KeyPixServiceImpl {
    repo: Arc<dyn KeyPixRepository>,
}
impl KeyPixServiceImpl {
    pub fn new(repo: Arc<dyn KeyPixRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl KeyPixService for KeyPixServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<KeyPix>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<KeyPix>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &KeyPix) -> Result<i64, ServiceError> {
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &KeyPix) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
