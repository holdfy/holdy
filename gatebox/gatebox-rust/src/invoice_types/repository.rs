use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use crate::model::InvoiceTypes;
use crate::shared::types::ItemsPage;
use super::ddl;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not found")]
    NotFound,
}

#[async_trait]
pub trait InvoiceTypesRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<InvoiceTypes>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<InvoiceTypes>, RepositoryError>;
    async fn insert(&self, item: &InvoiceTypes) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &InvoiceTypes) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct InvoiceTypesRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl InvoiceTypesRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    code: String,
    description: String,
}

#[async_trait]
impl InvoiceTypesRepository for InvoiceTypesRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<InvoiceTypes>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST).bind(limit).bind(offset).fetch_all(self.read.as_ref()).await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(|r| InvoiceTypes {
            id: r.id,
            code: r.code,
            description: r.description,
            full_count: None,
        }).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<InvoiceTypes>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID).bind(id).fetch_optional(self.read.as_ref()).await?;
        Ok(row.map(|r| InvoiceTypes {
            id: r.id,
            code: r.code,
            description: r.description,
            full_count: None,
        }))
    }
    async fn insert(&self, item: &InvoiceTypes) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT).bind(&item.code).bind(&item.description).fetch_one(self.write.as_ref()).await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &InvoiceTypes) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE).bind(&item.code).bind(&item.description).bind(id).execute(self.write.as_ref()).await?;
        if r.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError> {
        let r = sqlx::query(ddl::SQL_DELETE).bind(id).execute(self.write.as_ref()).await?;
        Ok(r.rows_affected() > 0)
    }
}
