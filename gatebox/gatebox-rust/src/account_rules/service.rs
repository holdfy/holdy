use async_trait::async_trait;
use std::sync::Arc;

use crate::model::AccountRules;
use crate::shared::types::ItemsPage;
use super::repository::{AccountRulesRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait AccountRulesService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<AccountRules>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<AccountRules>, ServiceError>;
    async fn create(&self, item: &AccountRules) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &AccountRules) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct AccountRulesServiceImpl {
    repo: Arc<dyn AccountRulesRepository>,
}
impl AccountRulesServiceImpl {
    pub fn new(repo: Arc<dyn AccountRulesRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AccountRulesService for AccountRulesServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<AccountRules>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<AccountRules>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &AccountRules) -> Result<i64, ServiceError> {
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &AccountRules) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
