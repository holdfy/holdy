use async_trait::async_trait;
use std::sync::Arc;

use crate::model::ControlMed;
use crate::shared::types::ItemsPage;
use super::repository::{ControlMedRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait ControlMedService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<ControlMed>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<ControlMed>, ServiceError>;
    async fn create(&self, item: &ControlMed) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &ControlMed) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct ControlMedServiceImpl {
    repo: Arc<dyn ControlMedRepository>,
}
impl ControlMedServiceImpl {
    pub fn new(repo: Arc<dyn ControlMedRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ControlMedService for ControlMedServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<ControlMed>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<ControlMed>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &ControlMed) -> Result<i64, ServiceError> {
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &ControlMed) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
