use async_trait::async_trait;
use std::sync::Arc;
use crate::model::Customer;
use crate::shared::types::ItemsPage;
use super::repository::{CustomerRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait CustomerService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Customer>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Customer>, ServiceError>;
    async fn create(&self, item: &Customer) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &Customer) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct CustomerServiceImpl { repo: Arc<dyn CustomerRepository> }
impl CustomerServiceImpl {
    pub fn new(repo: Arc<dyn CustomerRepository>) -> Self { Self { repo } }
}

#[async_trait]
impl CustomerService for CustomerServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Customer>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Customer>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &Customer) -> Result<i64, ServiceError> {
        if item.full_name.is_empty() { return Err(ServiceError::BadRequest("full_name required".into())); }
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &Customer) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
