use async_trait::async_trait;
use std::sync::Arc;
use crate::model::Fees;
use crate::shared::types::ItemsPage;
use super::repository::{FeesRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait FeesService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Fees>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Fees>, ServiceError>;
    async fn create(&self, item: &Fees) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &Fees) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct FeesServiceImpl { repo: Arc<dyn FeesRepository> }
impl FeesServiceImpl {
    pub fn new(repo: Arc<dyn FeesRepository>) -> Self { Self { repo } }
}

#[async_trait]
impl FeesService for FeesServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Fees>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Fees>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &Fees) -> Result<i64, ServiceError> {
        if item.account_id <= 0 { return Err(ServiceError::BadRequest("account_id required".to_string())); }
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &Fees) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
