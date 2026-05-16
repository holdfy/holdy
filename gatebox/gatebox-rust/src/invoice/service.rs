use async_trait::async_trait;
use std::sync::Arc;

use crate::model::Invoice;
use crate::shared::types::ItemsPage;
use super::repository::{RepositoryError, InvoiceRepository};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait InvoiceService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Invoice>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Invoice>, ServiceError>;
    async fn create(&self, item: &Invoice) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &Invoice) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct InvoiceServiceImpl {
    repo: Arc<dyn InvoiceRepository>,
}
impl InvoiceServiceImpl {
    pub fn new(repo: Arc<dyn InvoiceRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl InvoiceService for InvoiceServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Invoice>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Invoice>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &Invoice) -> Result<i64, ServiceError> {
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &Invoice) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
