use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use crate::model::PartnersList;
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
pub trait PartnersListRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<PartnersList>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<PartnersList>, RepositoryError>;
    async fn insert(&self, item: &PartnersList) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &PartnersList) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct PartnersListRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl PartnersListRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    description: String,
    site: String,
    contact: String,
    active: bool,
}

#[async_trait]
impl PartnersListRepository for PartnersListRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<PartnersList>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST).bind(limit).bind(offset).fetch_all(self.read.as_ref()).await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(|r| PartnersList {
            id: r.id,
            description: r.description,
            site: r.site,
            contact: r.contact,
            active: r.active,
            full_count: None,
        }).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<PartnersList>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID).bind(id).fetch_optional(self.read.as_ref()).await?;
        Ok(row.map(|r| PartnersList {
            id: r.id,
            description: r.description,
            site: r.site,
            contact: r.contact,
            active: r.active,
            full_count: None,
        }))
    }
    async fn insert(&self, item: &PartnersList) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.description)
            .bind(&item.site)
            .bind(&item.contact)
            .bind(item.active)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &PartnersList) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.description)
            .bind(&item.site)
            .bind(&item.contact)
            .bind(item.active)
            .bind(id)
            .execute(self.write.as_ref())
            .await?;
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
