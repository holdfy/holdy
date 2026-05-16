use async_trait::async_trait;
use std::sync::Arc;

use crate::model::TokenService;
use crate::shared::types::ItemsPage;
use super::repository::{RepositoryError, TokenServiceRepository};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait TokenServiceService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<TokenService>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<TokenService>, ServiceError>;
    async fn create(&self, item: &TokenService) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &TokenService) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct TokenServiceServiceImpl {
    repo: Arc<dyn TokenServiceRepository>,
}
impl TokenServiceServiceImpl {
    pub fn new(repo: Arc<dyn TokenServiceRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl TokenServiceService for TokenServiceServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<TokenService>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<TokenService>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &TokenService) -> Result<i64, ServiceError> {
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &TokenService) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
