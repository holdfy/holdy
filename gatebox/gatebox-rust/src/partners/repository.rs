use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use crate::model::Partners;
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
pub trait PartnersRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Partners>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Partners>, RepositoryError>;
    async fn get_by_authentication_id(&self, authentication_id: i64) -> Result<Option<Partners>, RepositoryError>;
    async fn insert(&self, item: &Partners) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &Partners) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub struct PartnersRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl PartnersRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    partners_list_id: i64,
    description: String,
    document: String,
    account: String,
    branch: String,
    authentication_id: i64,
    client_id: String,
    client_secret: String,
    authentication: String,
    password: String,
    whpix_in_id: String,
    whpix_out_id: String,
    type_authorize_id: i64,
    fixed_cash_in: Decimal,
    fixed_cash_out: Decimal,
    percent_cashin: Decimal,
    percent_cashout: Decimal,
    fixed_ref_cash_in: Decimal,
    fixed_ref_cash_out: Decimal,
    percent_ref_cashin: Decimal,
    percent_ref_cashout: Decimal,
    active: bool,
}

fn to_partners(r: Row) -> Partners {
    Partners {
        id: r.id,
        partners_list_id: r.partners_list_id,
        description: r.description,
        document: r.document,
        account: r.account,
        branch: r.branch,
        authentication_id: r.authentication_id,
        client_id: r.client_id,
        client_secret: r.client_secret,
        authentication: r.authentication,
        password: r.password,
        whpix_in_id: r.whpix_in_id,
        whpix_out_id: r.whpix_out_id,
        type_authorize_id: r.type_authorize_id,
        fixed_cash_in: r.fixed_cash_in,
        fixed_cash_out: r.fixed_cash_out,
        percent_cashin: r.percent_cashin,
        percent_cashout: r.percent_cashout,
        fixed_ref_cash_in: r.fixed_ref_cash_in,
        fixed_ref_cash_out: r.fixed_ref_cash_out,
        percent_ref_cashin: r.percent_ref_cashin,
        percent_ref_cashout: r.percent_ref_cashout,
        active: r.active,
        full_count: None,
    }
}

#[async_trait]
impl PartnersRepository for PartnersRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Partners>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST).bind(limit).bind(offset).fetch_all(self.read.as_ref()).await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_partners).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Partners>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID).bind(id).fetch_optional(self.read.as_ref()).await?;
        Ok(row.map(to_partners))
    }
    async fn get_by_authentication_id(&self, authentication_id: i64) -> Result<Option<Partners>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_AUTHENTICATION_ID)
            .bind(authentication_id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_partners))
    }
    async fn insert(&self, item: &Partners) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(item.partners_list_id)
            .bind(&item.description)
            .bind(&item.document)
            .bind(&item.account)
            .bind(&item.branch)
            .bind(item.authentication_id)
            .bind(&item.client_id)
            .bind(&item.client_secret)
            .bind(&item.authentication)
            .bind(&item.password)
            .bind(&item.whpix_in_id)
            .bind(&item.whpix_out_id)
            .bind(item.type_authorize_id)
            .bind(item.fixed_cash_in)
            .bind(item.fixed_cash_out)
            .bind(item.percent_cashin)
            .bind(item.percent_cashout)
            .bind(item.fixed_ref_cash_in)
            .bind(item.fixed_ref_cash_out)
            .bind(item.percent_ref_cashin)
            .bind(item.percent_ref_cashout)
            .bind(item.active)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &Partners) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(item.partners_list_id)
            .bind(&item.description)
            .bind(&item.document)
            .bind(&item.account)
            .bind(&item.branch)
            .bind(item.authentication_id)
            .bind(&item.client_id)
            .bind(&item.client_secret)
            .bind(&item.authentication)
            .bind(&item.password)
            .bind(&item.whpix_in_id)
            .bind(&item.whpix_out_id)
            .bind(item.type_authorize_id)
            .bind(item.fixed_cash_in)
            .bind(item.fixed_cash_out)
            .bind(item.percent_cashin)
            .bind(item.percent_cashout)
            .bind(item.fixed_ref_cash_in)
            .bind(item.fixed_ref_cash_out)
            .bind(item.percent_ref_cashin)
            .bind(item.percent_ref_cashout)
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
