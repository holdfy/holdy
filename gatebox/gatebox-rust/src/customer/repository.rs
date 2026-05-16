// Customer repository - converted from gateboxgo/app/customer/repository/customer.go
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use crate::model::Customer;
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
pub trait CustomerRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Customer>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Customer>, RepositoryError>;
    async fn get_by_authentication_id(&self, authentication_id: i64) -> Result<Option<Customer>, RepositoryError>;
    /// Search by email, phone, or document (for P2P recipient lookup).
    async fn search(&self, query: &str) -> Result<Vec<Customer>, RepositoryError>;
    async fn insert(&self, item: &Customer) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &Customer) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct CustomerRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl CustomerRepositoryImpl {
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
    company_id: i64,
    responsible_name: String,
    phone_number: String,
    email: String,
    customer_status_id: i64,
    is_politically_exposed_person: bool,
    authentication_id: i64,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

fn row_to_customer(r: Row) -> Customer {
    Customer {
        id: r.id,
        full_name: r.full_name,
        social_name: r.social_name,
        type_person_id: r.type_person_id,
        document_number: r.document_number,
        birth_date: r.birth_date,
        company_id: r.company_id,
        responsible_name: r.responsible_name,
        phone_number: r.phone_number,
        email: r.email,
        customer_status_id: r.customer_status_id,
        is_politically_exposed_person: r.is_politically_exposed_person,
        authentication_id: r.authentication_id,
        deleted_at: r.deleted_at,
        full_count: None,
    }
}

#[async_trait]
impl CustomerRepository for CustomerRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Customer>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(row_to_customer).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Customer>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(row_to_customer))
    }

    async fn get_by_authentication_id(&self, authentication_id: i64) -> Result<Option<Customer>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_AUTHENTICATION_ID)
            .bind(authentication_id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(row_to_customer))
    }

    async fn search(&self, query: &str) -> Result<Vec<Customer>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_SEARCH)
            .bind(query)
            .fetch_all(self.read.as_ref())
            .await?;
        Ok(rows.into_iter().map(row_to_customer).collect())
    }

    async fn insert(&self, item: &Customer) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.full_name)
            .bind(&item.social_name)
            .bind(item.type_person_id)
            .bind(&item.document_number)
            .bind(&item.birth_date)
            .bind(item.company_id)
            .bind(&item.responsible_name)
            .bind(&item.phone_number)
            .bind(&item.email)
            .bind(item.customer_status_id)
            .bind(item.is_politically_exposed_person)
            .bind(item.authentication_id)
            .bind(item.deleted_at)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }

    async fn update(&self, id: i64, item: &Customer) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.full_name)
            .bind(&item.social_name)
            .bind(item.type_person_id)
            .bind(&item.document_number)
            .bind(&item.birth_date)
            .bind(item.company_id)
            .bind(&item.responsible_name)
            .bind(&item.phone_number)
            .bind(&item.email)
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
        let r = sqlx::query(ddl::SQL_DELETE)
            .bind(id)
            .execute(self.write.as_ref())
            .await?;
        Ok(r.rows_affected() > 0)
    }
}
