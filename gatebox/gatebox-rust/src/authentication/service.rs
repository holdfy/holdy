use async_trait::async_trait;
use std::sync::Arc;
use crate::model::Authentication;
use crate::shared::types::ItemsPage;
use super::repository::{AuthenticationRepository, RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

/// type_auth_id: 1=CUSTOMER, 2=ADMIN, 3=MANAGER
pub const TYPE_AUTH_ADMIN: i64 = 2;

#[async_trait]
pub trait AuthenticationService: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Authentication>>, ServiceError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Authentication>, ServiceError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<Authentication>, ServiceError>;
    async fn find_by_username_and_type(&self, username: &str, type_auth_id: i64) -> Result<Option<Authentication>, ServiceError>;
    async fn update_password(&self, id: i64, password: &str) -> Result<(), ServiceError>;
    async fn create(&self, item: &Authentication) -> Result<i64, ServiceError>;
    async fn update(&self, id: i64, item: &Authentication) -> Result<(), ServiceError>;
    async fn delete(&self, id: i64) -> Result<bool, ServiceError>;
}

pub struct AuthenticationServiceImpl { repo: Arc<dyn AuthenticationRepository> }
impl AuthenticationServiceImpl {
    pub fn new(repo: Arc<dyn AuthenticationRepository>) -> Self { Self { repo } }
}

#[async_trait]
impl AuthenticationService for AuthenticationServiceImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Authentication>>, ServiceError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);
        self.repo.list(offset, limit).await.map_err(ServiceError::Repository)
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Authentication>, ServiceError> {
        self.repo.get_by_id(id).await.map_err(ServiceError::Repository)
    }
    async fn find_by_username(&self, username: &str) -> Result<Option<Authentication>, ServiceError> {
        self.repo.get_by_username(username).await.map_err(ServiceError::Repository)
    }
    async fn find_by_username_and_type(&self, username: &str, type_auth_id: i64) -> Result<Option<Authentication>, ServiceError> {
        self.repo.get_by_username_and_type(username, type_auth_id).await.map_err(ServiceError::Repository)
    }
    async fn update_password(&self, id: i64, password: &str) -> Result<(), ServiceError> {
        self.repo.update_password(id, password).await.map_err(ServiceError::Repository)
    }
    async fn create(&self, item: &Authentication) -> Result<i64, ServiceError> {
        if item.username.is_empty() { return Err(ServiceError::BadRequest("username required".to_string())); }
        self.repo.insert(item).await.map_err(ServiceError::Repository)
    }
    async fn update(&self, id: i64, item: &Authentication) -> Result<(), ServiceError> {
        self.repo.update(id, item).await.map_err(ServiceError::Repository)
    }
    async fn delete(&self, id: i64) -> Result<bool, ServiceError> {
        self.repo.delete(id).await.map_err(ServiceError::Repository)
    }
}
