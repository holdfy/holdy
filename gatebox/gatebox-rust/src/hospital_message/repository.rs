use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

use super::ddl;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database: {0}")]
    Database(#[from] sqlx::Error),
}

#[async_trait]
pub trait HospitalMessageRepository: Send + Sync {
    async fn insert(
        &self,
        payment_id: &str,
        amount: Decimal,
        retry_count: i32,
        payload_json: &str,
    ) -> Result<i64, RepositoryError>;
}

pub struct HospitalMessageRepositoryImpl {
    write: Arc<PgPool>,
}

impl HospitalMessageRepositoryImpl {
    pub fn new(write: Arc<PgPool>) -> Self {
        Self { write }
    }
}

#[async_trait]
impl HospitalMessageRepository for HospitalMessageRepositoryImpl {
    async fn insert(
        &self,
        payment_id: &str,
        amount: Decimal,
        retry_count: i32,
        payload_json: &str,
    ) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(payment_id)
            .bind(amount)
            .bind(retry_count)
            .bind(payload_json)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
}
