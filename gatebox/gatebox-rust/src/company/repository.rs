use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use crate::model::Company;
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
pub trait CompanyRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Company>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Company>, RepositoryError>;
    async fn insert(&self, item: &Company) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &Company) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct CompanyRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl CompanyRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    full_name: String,
    social_name: String,
    type_person_id: i64,
    document_number: String,
    birth_date: String,
    responsible_name: String,
    phone_number: String,
    email: String,
    telegram_chat_id: String,
    domanin: String,
    customer_status_id: i64,
    is_politically_exposed_person: bool,
    authentication_id: i64,
    deleted_at: Option<DateTime<Utc>>,
}

fn to_company(r: Row) -> Company {
    Company {
        id: r.id,
        full_name: r.full_name,
        social_name: r.social_name,
        type_person_id: r.type_person_id,
        document_number: r.document_number,
        birth_date: r.birth_date,
        responsible_name: r.responsible_name,
        phone_number: r.phone_number,
        email: r.email,
        telegram_chat_id: r.telegram_chat_id,
        domanin: r.domanin,
        customer_status_id: r.customer_status_id,
        is_politically_exposed_person: r.is_politically_exposed_person,
        authentication_id: r.authentication_id,
        deleted_at: r.deleted_at,
        full_count: None,
    }
}

#[async_trait]
impl CompanyRepository for CompanyRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Company>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST).bind(limit).bind(offset).fetch_all(self.read.as_ref()).await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_company).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Company>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID).bind(id).fetch_optional(self.read.as_ref()).await?;
        Ok(row.map(to_company))
    }
    async fn insert(&self, item: &Company) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.full_name)
            .bind(&item.social_name)
            .bind(item.type_person_id)
            .bind(&item.document_number)
            .bind(&item.birth_date)
            .bind(&item.responsible_name)
            .bind(&item.phone_number)
            .bind(&item.email)
            .bind(&item.telegram_chat_id)
            .bind(&item.domanin)
            .bind(item.customer_status_id)
            .bind(item.is_politically_exposed_person)
            .bind(item.authentication_id)
            .bind(item.deleted_at)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &Company) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.full_name)
            .bind(&item.social_name)
            .bind(item.type_person_id)
            .bind(&item.document_number)
            .bind(&item.birth_date)
            .bind(&item.responsible_name)
            .bind(&item.phone_number)
            .bind(&item.email)
            .bind(&item.telegram_chat_id)
            .bind(&item.domanin)
            .bind(item.customer_status_id)
            .bind(item.is_politically_exposed_person)
            .bind(item.authentication_id)
            .bind(item.deleted_at)
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
