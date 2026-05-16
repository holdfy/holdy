use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use crate::model::Address;
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
pub trait AddressRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Address>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Address>, RepositoryError>;
    async fn insert(&self, item: &Address) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &Address) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct AddressRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl AddressRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    postal_code: String,
    street: String,
    number: String,
    address_complement: String,
    neighborhood: String,
    city: String,
    state: String,
    address_type_id: i64,
    customer_id: i64,
    business_id: i64,
    deleted_at: Option<DateTime<Utc>>,
    company_id: i64,
}

fn to_address(r: Row) -> Address {
    Address {
        id: r.id,
        postal_code: r.postal_code,
        street: r.street,
        number: r.number,
        address_complement: r.address_complement,
        neighborhood: r.neighborhood,
        city: r.city,
        state: r.state,
        address_type_id: r.address_type_id,
        customer_id: r.customer_id,
        business_id: r.business_id,
        deleted_at: r.deleted_at,
        company_id: r.company_id,
        full_count: None,
    }
}

#[async_trait]
impl AddressRepository for AddressRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Address>>, RepositoryError> {
        let rows: Vec<Row> =
            sqlx::query_as(ddl::SQL_LIST).bind(limit).bind(offset).fetch_all(self.read.as_ref()).await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_address).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Address>, RepositoryError> {
        let row: Option<Row> =
            sqlx::query_as(ddl::SQL_GET_BY_ID).bind(id).fetch_optional(self.read.as_ref()).await?;
        Ok(row.map(to_address))
    }
    async fn insert(&self, item: &Address) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(&item.postal_code)
            .bind(&item.street)
            .bind(&item.number)
            .bind(&item.address_complement)
            .bind(&item.neighborhood)
            .bind(&item.city)
            .bind(&item.state)
            .bind(item.address_type_id)
            .bind(item.customer_id)
            .bind(item.business_id)
            .bind(item.deleted_at)
            .bind(item.company_id)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &Address) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(&item.postal_code)
            .bind(&item.street)
            .bind(&item.number)
            .bind(&item.address_complement)
            .bind(&item.neighborhood)
            .bind(&item.city)
            .bind(&item.state)
            .bind(item.address_type_id)
            .bind(item.customer_id)
            .bind(item.business_id)
            .bind(item.deleted_at)
            .bind(item.company_id)
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
