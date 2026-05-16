use async_trait::async_trait;
use std::sync::Arc;

use crate::model::WithListAccounts;
use crate::shared::types::ItemsPage;
use super::repository::{RepositoryError, WithListAccountsRepository};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait WithListAccountsService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<WithListAccounts>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<WithListAccounts>, ServiceError>;
    async fn create(&self, item: &WithListAccounts) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &WithListAccounts) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct WithListAccountsServiceImpl {
    repo: Arc<dyn WithListAccountsRepository>,
}
impl WithListAccountsServiceImpl {
    pub fn new(repo: Arc<dyn WithListAccountsRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl WithListAccountsService for WithListAccountsServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<WithListAccounts>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<WithListAccounts>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &WithListAccounts) -> Result<i64, ServiceError> {
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &WithListAccounts) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
