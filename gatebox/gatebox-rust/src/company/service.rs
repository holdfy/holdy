use async_trait::async_trait;
use std::sync::Arc;
use crate::model::Company;
use crate::shared::types::ItemsPage;
use super::repository::{CompanyRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait CompanyService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Company>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Company>, ServiceError>;
    async fn create(&self, item: &Company) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &Company) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct CompanyServiceImpl { repo: Arc<dyn CompanyRepository> }
impl CompanyServiceImpl {
    pub fn new(repo: Arc<dyn CompanyRepository>) -> Self { Self { repo } }
}

#[async_trait]
impl CompanyService for CompanyServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Company>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Company>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &Company) -> Result<i64, ServiceError> {
        if item.full_name.is_empty() { return Err(ServiceError::BadRequest("full_name required".to_string())); }
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &Company) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
