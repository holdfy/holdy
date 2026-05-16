use async_trait::async_trait;
use std::sync::Arc;

use crate::model::WebhookManager;
use crate::shared::types::ItemsPage;
use super::repository::{RepositoryError, WebhookManagerRepository};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait WebhookManagerService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<WebhookManager>>, ServiceError>;
    async fn list_by_account(&self, account_id: i64, offset: i64, limit: i64) -> Result<Vec<WebhookManager>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<WebhookManager>, ServiceError>;
    async fn create(&self, item: &WebhookManager) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &WebhookManager) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct WebhookManagerServiceImpl {
    repo: Arc<dyn WebhookManagerRepository>,
}
impl WebhookManagerServiceImpl {
    pub fn new(repo: Arc<dyn WebhookManagerRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl WebhookManagerService for WebhookManagerServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<WebhookManager>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn list_by_account(&self, account_id: i64, offset: i64, limit: i64) -> Result<Vec<WebhookManager>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list_by_account(account_id, offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<WebhookManager>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &WebhookManager) -> Result<i64, ServiceError> {
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &WebhookManager) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
