use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::Styled;
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
pub trait StyledRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Styled>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Styled>, RepositoryError>;
    async fn insert(&self, item: &Styled) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &Styled) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct StyledRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl StyledRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    url: String,
    application_name: String,
    title: String,
    primary_color: String,
    secondary_color: String,
    font_color: String,
    img: String,
    favicon: String,
    styled_type_id: i64,
    company_id: i64,
    active: bool,
}

fn to_styled(r: Row) -> Styled {
    Styled {
        id: r.id,
        url: r.url,
        application_name: r.application_name,
        title: r.title,
        primary_color: r.primary_color,
        secondary_color: r.secondary_color,
        font_color: r.font_color,
        img: r.img,
        favicon: r.favicon,
        styled_type_id: r.styled_type_id,
        company_id: r.company_id,
        active: r.active,
        full_count: None,
    }
}

#[async_trait]
impl StyledRepository for StyledRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Styled>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_styled).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Styled>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_styled))
    }
    async fn insert(&self, item: &Styled) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.url)
            .bind(&item.application_name)
            .bind(&item.title)
            .bind(&item.primary_color)
            .bind(&item.secondary_color)
            .bind(&item.font_color)
            .bind(&item.img)
            .bind(&item.favicon)
            .bind(item.styled_type_id)
            .bind(item.company_id)
            .bind(item.active)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &Styled) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.url)
            .bind(&item.application_name)
            .bind(&item.title)
            .bind(&item.primary_color)
            .bind(&item.secondary_color)
            .bind(&item.font_color)
            .bind(&item.img)
            .bind(&item.favicon)
            .bind(item.styled_type_id)
            .bind(item.company_id)
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
