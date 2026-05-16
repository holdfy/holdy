use async_trait::async_trait;
use std::sync::Arc;
use crate::model::StatusSecMedTypes;
use crate::shared::types::ItemsPage;
use super::repository::{StatusSecMedTypesRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait StatusSecMedTypesService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<StatusSecMedTypes>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<StatusSecMedTypes>, ServiceError>;
    async fn create(&self, item: &StatusSecMedTypes) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &StatusSecMedTypes) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct StatusSecMedTypesServiceImpl {
    repo: Arc<dyn StatusSecMedTypesRepository>,
}

impl StatusSecMedTypesServiceImpl {
    pub fn new(repo: Arc<dyn StatusSecMedTypesRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl StatusSecMedTypesService for StatusSecMedTypesServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<StatusSecMedTypes>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<StatusSecMedTypes>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &StatusSecMedTypes) -> Result<i64, ServiceError> {
        if item.code.is_empty() {
            return Err(ServiceError::BadRequest("code required".to_string()));
        }
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &StatusSecMedTypes) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
