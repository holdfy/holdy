use async_trait::async_trait;
use std::sync::Arc;
use crate::model::WebhookTypes;
use crate::shared::types::ItemsPage;
use super::repository::{WebhookTypesRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait WebhookTypesService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<WebhookTypes>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<WebhookTypes>, ServiceError>;
    async fn create(&self, item: &WebhookTypes) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &WebhookTypes) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct WebhookTypesServiceImpl {
    repo: Arc<dyn WebhookTypesRepository>,
}

impl WebhookTypesServiceImpl {
    pub fn new(repo: Arc<dyn WebhookTypesRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl WebhookTypesService for WebhookTypesServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<WebhookTypes>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<WebhookTypes>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &WebhookTypes) -> Result<i64, ServiceError> {
        if item.code.is_empty() {
            return Err(ServiceError::BadRequest("code required".to_string()));
        }
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &WebhookTypes) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
