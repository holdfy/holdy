use async_trait::async_trait;
use std::sync::Arc;

use crate::model::Transaction;
use crate::shared::types::ItemsPage;
use super::repository::{CustomerActivityRow, RepositoryError, TransactionRepository};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[async_trait]
pub trait TransactionService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Transaction>>, ServiceError>;
    async fn list_by_account(
        &self,
        account_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<Transaction>>, ServiceError>;
    async fn get_balance(&self, account_id: i64) -> Result<rust_decimal::Decimal, ServiceError>;
    async fn get_profit(&self, admin_account_id: i64) -> Result<rust_decimal::Decimal, ServiceError>;
    async fn get_customer_activities(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<CustomerActivityRow>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Transaction>, ServiceError>;
    async fn create(&self, item: &Transaction) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &Transaction) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct TransactionServiceImpl {
    repo: Arc<dyn TransactionRepository>,
}
impl TransactionServiceImpl {
    pub fn new(repo: Arc<dyn TransactionRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl TransactionService for TransactionServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Transaction>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn list_by_account(
        &self,
        account_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<Transaction>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo
            .list_by_account(account_id, offset, limit)
            .await
            .map_err(ServiceError::Repository)
    }
    async fn get_balance(&self, account_id: i64) -> Result<rust_decimal::Decimal, ServiceError> {
        self.repo.get_balance(account_id).await.map_err(ServiceError::Repository)
    }
    async fn get_profit(&self, admin_account_id: i64) -> Result<rust_decimal::Decimal, ServiceError> {
        self.repo.get_profit(admin_account_id).await.map_err(ServiceError::Repository)
    }
    async fn get_customer_activities(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<CustomerActivityRow>>, ServiceError> {
        let limit = limit.clamp(1, 200);
        let offset = offset.max(0);
        self.repo
            .get_customer_activities(offset, limit)
            .await
            .map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Transaction>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &Transaction) -> Result<i64, ServiceError> {
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &Transaction) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
