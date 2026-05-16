use async_trait::async_trait;
use std::sync::Arc;
use crate::model::Address;
use crate::shared::types::ItemsPage;
use super::repository::{AddressRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait AddressService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Address>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Address>, ServiceError>;
    async fn create(&self, item: &Address) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &Address) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct AddressServiceImpl {
    repo: Arc<dyn AddressRepository>,
}

impl AddressServiceImpl {
    pub fn new(repo: Arc<dyn AddressRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AddressService for AddressServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Address>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Address>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &Address) -> Result<i64, ServiceError> {
        if item.postal_code.is_empty() {
            return Err(ServiceError::BadRequest("postal_code required".to_string()));
        }
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &Address) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
