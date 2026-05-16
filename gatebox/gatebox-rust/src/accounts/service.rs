use async_trait::async_trait;
use std::sync::Arc;

use crate::model::Accounts;
use crate::shared::types::ItemsPage;

use super::repository::{AccountsRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait AccountsService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Accounts>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Accounts>, ServiceError>;
    async fn get_by_authentication_id(&self, authentication_id: i64) -> Result<Option<Accounts>, ServiceError>;
    async fn create(&self, item: &Accounts) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &Accounts) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct AccountsServiceImpl {
    repo: Arc<dyn AccountsRepository>,
}

impl AccountsServiceImpl {
    pub fn new(repo: Arc<dyn AccountsRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AccountsService for AccountsServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Accounts>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Accounts>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }

    async fn get_by_authentication_id(&self, authentication_id: i64) -> Result<Option<Accounts>, ServiceError> {
        self.repo
            .get_by_authentication_id(authentication_id)
            .await
            .map_err(ServiceError::Repository)
    }

    async fn create(&self, item: &Accounts) -> Result<i64, ServiceError> {
        if item.account_number.is_empty() {
            return Err(ServiceError::BadRequest("account_number is required".to_string()));
        }
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }

    async fn update(&self, id: i64, item: &Accounts) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }

    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
