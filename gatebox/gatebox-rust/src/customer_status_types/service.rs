use async_trait::async_trait;
use std::sync::Arc;
use crate::model::CustomerStatusTypes;
use crate::shared::types::ItemsPage;
use super::repository::{CustomerStatusTypesRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait CustomerStatusTypesService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<CustomerStatusTypes>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<CustomerStatusTypes>, ServiceError>;
    async fn get_by_code(&self, code: &str) -> Result<Option<CustomerStatusTypes>, ServiceError>;
    async fn create(&self, item: &CustomerStatusTypes) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &CustomerStatusTypes) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct CustomerStatusTypesServiceImpl {
    repo: Arc<dyn CustomerStatusTypesRepository>,
}

impl CustomerStatusTypesServiceImpl {
    pub fn new(repo: Arc<dyn CustomerStatusTypesRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl CustomerStatusTypesService for CustomerStatusTypesServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<CustomerStatusTypes>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<CustomerStatusTypes>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn get_by_code(&self, code: &str) -> Result<Option<CustomerStatusTypes>, ServiceError> {
        self.repo.get_by_code(code).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &CustomerStatusTypes) -> Result<i64, ServiceError> {
        if item.code.is_empty() {
            return Err(ServiceError::BadRequest("code required".to_string()));
        }
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &CustomerStatusTypes) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
