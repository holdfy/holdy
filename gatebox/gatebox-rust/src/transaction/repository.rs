use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

use crate::model::Transaction;
use crate::shared::types::ItemsPage;
use super::ddl;

/// Row for customer activities report.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CustomerActivityRow {
    pub customer_id: i64,
    pub full_name: Option<String>,
    pub tx_count: i64,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not found")]
    NotFound,
}

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Transaction>>, RepositoryError>;
    /// List transactions by account_id (for admin customer extract).
    async fn list_by_account(
        &self,
        account_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<Transaction>>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Transaction>, RepositoryError>;
    async fn insert(&self, item: &Transaction) -> Result<i64, RepositoryError>;
    async fn update(&self, id: i64, item: &Transaction) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
    /// Update PIX status (status_transaction_id, msg_error, gateway) for worker processing.
    async fn update_pix_status(
        &self,
        id: i64,
        status_transaction_id: i64,
        msg_error: &str,
        gateway: &str,
    ) -> Result<(), RepositoryError>;
    /// Update PIX status and endtoend_id (when gateway returns endToEndId for reversal).
    async fn update_pix_status_with_endtoend(
        &self,
        id: i64,
        status_transaction_id: i64,
        msg_error: &str,
        gateway: &str,
        endtoend_id: &str,
    ) -> Result<(), RepositoryError>;
    /// Get balance for account (sum of CREDIT - DEBIT for status 3,4).
    async fn get_balance(&self, account_id: i64) -> Result<rust_decimal::Decimal, RepositoryError>;
    /// Get profit (TTO+TPO credits) for admin account.
    async fn get_profit(&self, admin_account_id: i64) -> Result<rust_decimal::Decimal, RepositoryError>;
    /// Check idempotency: existing completed tx with same external_id, account, amount today.
    async fn find_duplicate_external_id(
        &self,
        external_id: &str,
        account_id: i64,
        amount: rust_decimal::Decimal,
        today_start: chrono::DateTime<chrono::Utc>,
        today_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<i64>, RepositoryError>;
    /// PIX IN idempotency: existing CREDIT tx with same external_id or idempotency_key.
    async fn find_pix_in_duplicate(&self, end_to_end_id: &str, idempotency_key: &str) -> Result<Option<i64>, RepositoryError>;
    /// Find transaction id by external_id (for ReceivePixOut).
    async fn find_id_by_external_id(&self, external_id: &str) -> Result<Option<i64>, RepositoryError>;
    /// Find original PIX OUT for reversal: (id, account_id, amount).
    async fn find_original_for_reversal(&self, endtoend_id: &str) -> Result<Option<(i64, i64, Decimal)>, RepositoryError>;
    /// Check reversal idempotency: existing tx with external_id and account_id.
    async fn find_reversal_duplicate(&self, external_id: &str, account_id: i64) -> Result<Option<i64>, RepositoryError>;
    /// Insert TTO (sub_type=5). Returns new id.
    async fn insert_tto(
        &self,
        account_id: i64,
        parent_id: i64,
        endtoend_id: &str,
        name: &str,
        document_number: &str,
        amount: rust_decimal::Decimal,
        type_transaction_id: i64,
        description: &str,
    ) -> Result<i64, RepositoryError>;
    /// Insert TPO (sub_type=6). Returns new id.
    async fn insert_tpo(
        &self,
        account_id: i64,
        partners_id: i64,
        parent_id: i64,
        endtoend_id: &str,
        name: &str,
        amount: rust_decimal::Decimal,
        description: &str,
    ) -> Result<i64, RepositoryError>;
    /// Insert SMD (sub_type=7). Returns new id.
    async fn insert_smd(
        &self,
        account_id: i64,
        parent_id: i64,
        endtoend_id: &str,
        name: &str,
        document_number: &str,
        amount: rust_decimal::Decimal,
        description: &str,
    ) -> Result<i64, RepositoryError>;
    /// Insert P2P debit (sub_type=3). Returns new id.
    async fn insert_p2p_debit(
        &self,
        account_id: i64,
        parent_id: i64,
        external_id: &str,
        name: &str,
        document_number: &str,
        amount: Decimal,
        description: &str,
    ) -> Result<i64, RepositoryError>;
    /// Insert P2P credit (sub_type=3). Returns new id.
    async fn insert_p2p_credit(
        &self,
        account_id: i64,
        parent_id: i64,
        external_id: &str,
        name: &str,
        document_number: &str,
        amount: Decimal,
        description: &str,
    ) -> Result<i64, RepositoryError>;
    /// List P2P transactions for account (sub_type=3).
    async fn list_p2p_by_account(
        &self,
        account_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Transaction>, RepositoryError>;
    /// Execute P2P transfer atomically: debit sender, credit receiver. Returns (debit_id, credit_id).
    async fn insert_p2p_transfer(
        &self,
        sender_account_id: i64,
        receiver_account_id: i64,
        external_id: &str,
        sender_name: &str,
        sender_document: &str,
        receiver_name: &str,
        receiver_document: &str,
        amount: Decimal,
        description: &str,
    ) -> Result<(i64, i64), RepositoryError>;
    /// Customer activities: customers with tx_count and last_activity.
    async fn get_customer_activities(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<CustomerActivityRow>>, RepositoryError>;
    /// Insert PIX IN credit (type=2, sub_type=1) with gateway, pix_operation_type and fee audit fields.
    async fn insert_pix_in_credit(
        &self,
        account_id: i64,
        invoice_id: i64,
        partners_id: i64,
        external_id: &str,
        name: &str,
        document_number: &str,
        amount: Decimal,
        key: &str,
        remittance_information: &str,
        gateway: &str,
        pix_operation_type: &str,
        requested_amount: Decimal,
        net_amount: Decimal,
        total_amount: Decimal,
        fee_fixed: Decimal,
        fee_percent_rate: Decimal,
        fee_percent_amount: Decimal,
        fee_total: Decimal,
        partner_fixed_cash_in: Decimal,
        partner_percent_cashin: Decimal,
    ) -> Result<i64, RepositoryError>;
}

pub struct TransactionRepositoryImpl {
    read: Arc<PgPool>,
    write: Arc<PgPool>,
}

impl TransactionRepositoryImpl {
    pub fn new(read: Arc<PgPool>, write: Arc<PgPool>) -> Self {
        Self { read, write }
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    account_id: i64,
    invoice_id: i64,
    partners_id: i64,
    transaction_id: String,
    charger_back_id: String,
    parent_id: i64,
    external_id: String,
    name: String,
    email: String,
    document_number: String,
    description: String,
    phone: String,
    amount: Decimal,
    isbp: String,
    bank_name: String,
    branch: String,
    account: String,
    endtoend_id: String,
    pix_key_type_id: i64,
    key: String,
    type_transaction_id: i64,
    sub_type_transaction_id: i64,
    remittance_information: String,
    status_transaction_id: i64,
    msg_error: String,
    telegram_notification: bool,
    try_count: i64,
    deleted_at: Option<DateTime<Utc>>,
    endtoend_id_temp: String,
}

fn to_transaction(r: Row) -> Transaction {
    Transaction {
        id: r.id,
        account_id: r.account_id,
        invoice_id: r.invoice_id,
        partners_id: r.partners_id,
        transaction_id: r.transaction_id,
        charger_back_id: r.charger_back_id,
        parent_id: r.parent_id,
        external_id: r.external_id,
        name: r.name,
        email: r.email,
        document_number: r.document_number,
        description: r.description,
        phone: r.phone,
        amount: r.amount,
        isbp: r.isbp,
        bank_name: r.bank_name,
        branch: r.branch,
        account: r.account,
        endtoend_id: r.endtoend_id,
        pix_key_type_id: r.pix_key_type_id,
        key: r.key,
        type_transaction_id: r.type_transaction_id,
        sub_type_transaction_id: r.sub_type_transaction_id,
        remittance_information: r.remittance_information,
        status_transaction_id: r.status_transaction_id,
        msg_error: r.msg_error,
        telegram_notification: r.telegram_notification,
        try_count: r.try_count,
        deleted_at: r.deleted_at,
        endtoend_id_temp: r.endtoend_id_temp,
        full_count: None,
    }
}

#[async_trait]
impl TransactionRepository for TransactionRepositoryImpl {
    async fn list(&self, offset: i64, limit: i64) -> Result<ItemsPage<Vec<Transaction>>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total = rows.len() as i64;
        let items = rows.into_iter().map(to_transaction).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn list_by_account(
        &self,
        account_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<Transaction>>, RepositoryError> {
        let (total,): (i64,) = sqlx::query_as(ddl::SQL_COUNT_BY_ACCOUNT)
            .bind(account_id)
            .fetch_one(self.read.as_ref())
            .await?;
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST_BY_ACCOUNT)
            .bind(account_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let items = rows.into_iter().map(to_transaction).collect();
        Ok(ItemsPage { offset, limit, total, items })
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Transaction>, RepositoryError> {
        let row: Option<Row> = sqlx::query_as(ddl::SQL_GET_BY_ID)
            .bind(id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(to_transaction))
    }
    async fn insert(&self, item: &Transaction) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT)
            .bind(item.account_id)
            .bind(item.invoice_id)
            .bind(item.partners_id)
            .bind(&item.transaction_id)
            .bind(&item.charger_back_id)
            .bind(item.parent_id)
            .bind(&item.external_id)
            .bind(&item.name)
            .bind(&item.email)
            .bind(&item.document_number)
            .bind(&item.description)
            .bind(&item.phone)
            .bind(item.amount)
            .bind(&item.isbp)
            .bind(&item.bank_name)
            .bind(&item.branch)
            .bind(&item.account)
            .bind(&item.endtoend_id)
            .bind(item.pix_key_type_id)
            .bind(&item.key)
            .bind(item.type_transaction_id)
            .bind(item.sub_type_transaction_id)
            .bind(&item.remittance_information)
            .bind(item.status_transaction_id)
            .bind(&item.msg_error)
            .bind(item.telegram_notification)
            .bind(item.try_count)
            .bind(item.deleted_at)
            .bind(&item.endtoend_id_temp)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
    async fn update(&self, id: i64, item: &Transaction) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE)
            .bind(item.account_id)
            .bind(item.invoice_id)
            .bind(item.partners_id)
            .bind(&item.transaction_id)
            .bind(&item.charger_back_id)
            .bind(item.parent_id)
            .bind(&item.external_id)
            .bind(&item.name)
            .bind(&item.email)
            .bind(&item.document_number)
            .bind(&item.description)
            .bind(&item.phone)
            .bind(item.amount)
            .bind(&item.isbp)
            .bind(&item.bank_name)
            .bind(&item.branch)
            .bind(&item.account)
            .bind(&item.endtoend_id)
            .bind(item.pix_key_type_id)
            .bind(&item.key)
            .bind(item.type_transaction_id)
            .bind(item.sub_type_transaction_id)
            .bind(&item.remittance_information)
            .bind(item.status_transaction_id)
            .bind(&item.msg_error)
            .bind(item.telegram_notification)
            .bind(item.try_count)
            .bind(item.deleted_at)
            .bind(&item.endtoend_id_temp)
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

    async fn update_pix_status(
        &self,
        id: i64,
        status_transaction_id: i64,
        msg_error: &str,
        gateway: &str,
    ) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE_PIX_STATUS)
            .bind(status_transaction_id)
            .bind(msg_error)
            .bind(gateway)
            .bind(id)
            .execute(self.write.as_ref())
            .await?;
        if r.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn update_pix_status_with_endtoend(
        &self,
        id: i64,
        status_transaction_id: i64,
        msg_error: &str,
        gateway: &str,
        endtoend_id: &str,
    ) -> Result<(), RepositoryError> {
        let r = sqlx::query(ddl::SQL_UPDATE_PIX_STATUS_ENDTOEND)
            .bind(status_transaction_id)
            .bind(msg_error)
            .bind(gateway)
            .bind(endtoend_id)
            .bind(id)
            .execute(self.write.as_ref())
            .await?;
        if r.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn get_balance(&self, account_id: i64) -> Result<rust_decimal::Decimal, RepositoryError> {
        let row: (Decimal,) = sqlx::query_as(ddl::SQL_BALANCE)
            .bind(account_id)
            .fetch_one(self.read.as_ref())
            .await?;
        Ok(row.0)
    }

    async fn get_profit(&self, admin_account_id: i64) -> Result<rust_decimal::Decimal, RepositoryError> {
        let row: (Decimal,) = sqlx::query_as(ddl::SQL_PROFIT)
            .bind(admin_account_id)
            .fetch_one(self.read.as_ref())
            .await?;
        Ok(row.0)
    }

    async fn find_duplicate_external_id(
        &self,
        external_id: &str,
        account_id: i64,
        amount: rust_decimal::Decimal,
        today_start: chrono::DateTime<chrono::Utc>,
        today_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<i64>, RepositoryError> {
        let row: Option<(i64,)> = sqlx::query_as(ddl::SQL_IDEMPOTENCY_CHECK)
            .bind(external_id)
            .bind(account_id)
            .bind(amount)
            .bind(today_start)
            .bind(today_end)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(|r| r.0))
    }

    async fn find_pix_in_duplicate(&self, end_to_end_id: &str, idempotency_key: &str) -> Result<Option<i64>, RepositoryError> {
        let row: Option<(i64,)> = sqlx::query_as(ddl::SQL_PIX_IN_IDEMPOTENCY)
            .bind(end_to_end_id)
            .bind(idempotency_key)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(|r| r.0))
    }

    async fn find_id_by_external_id(&self, external_id: &str) -> Result<Option<i64>, RepositoryError> {
        let row: Option<(i64,)> = sqlx::query_as(ddl::SQL_FIND_BY_EXTERNAL_ID)
            .bind(external_id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(|r| r.0))
    }

    async fn find_original_for_reversal(&self, endtoend_id: &str) -> Result<Option<(i64, i64, Decimal)>, RepositoryError> {
        let row: Option<(i64, i64, Decimal)> = sqlx::query_as(ddl::SQL_FIND_ORIGINAL_FOR_REVERSAL)
            .bind(endtoend_id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row)
    }

    async fn find_reversal_duplicate(&self, external_id: &str, account_id: i64) -> Result<Option<i64>, RepositoryError> {
        let row: Option<(i64,)> = sqlx::query_as(ddl::SQL_REVERSAL_IDEMPOTENCY)
            .bind(external_id)
            .bind(account_id)
            .fetch_optional(self.read.as_ref())
            .await?;
        Ok(row.map(|r| r.0))
    }

    async fn insert_tto(
        &self,
        account_id: i64,
        parent_id: i64,
        endtoend_id: &str,
        name: &str,
        document_number: &str,
        amount: Decimal,
        type_transaction_id: i64,
        description: &str,
    ) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT_TTO)
            .bind(account_id)
            .bind(parent_id)
            .bind(endtoend_id)
            .bind(name)
            .bind(document_number)
            .bind(amount)
            .bind(type_transaction_id)
            .bind(description)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }

    async fn insert_tpo(
        &self,
        account_id: i64,
        partners_id: i64,
        parent_id: i64,
        endtoend_id: &str,
        name: &str,
        amount: Decimal,
        description: &str,
    ) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT_TPO)
            .bind(account_id)
            .bind(partners_id)
            .bind(parent_id)
            .bind(endtoend_id)
            .bind(name)
            .bind(amount)
            .bind(description)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }

    async fn insert_smd(
        &self,
        account_id: i64,
        parent_id: i64,
        endtoend_id: &str,
        name: &str,
        document_number: &str,
        amount: Decimal,
        description: &str,
    ) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT_SMD)
            .bind(account_id)
            .bind(parent_id)
            .bind(endtoend_id)
            .bind(name)
            .bind(document_number)
            .bind(amount)
            .bind(description)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }

    async fn insert_p2p_debit(
        &self,
        account_id: i64,
        parent_id: i64,
        external_id: &str,
        name: &str,
        document_number: &str,
        amount: Decimal,
        description: &str,
    ) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT_P2P_DEBIT)
            .bind(account_id)
            .bind(parent_id)
            .bind(external_id)
            .bind(name)
            .bind(document_number)
            .bind(amount)
            .bind(description)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }

    async fn insert_p2p_credit(
        &self,
        account_id: i64,
        parent_id: i64,
        external_id: &str,
        name: &str,
        document_number: &str,
        amount: Decimal,
        description: &str,
    ) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT_P2P_CREDIT)
            .bind(account_id)
            .bind(parent_id)
            .bind(external_id)
            .bind(name)
            .bind(document_number)
            .bind(amount)
            .bind(description)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }

    async fn list_p2p_by_account(
        &self,
        account_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Transaction>, RepositoryError> {
        let rows: Vec<Row> = sqlx::query_as(ddl::SQL_LIST_P2P_BY_ACCOUNT)
            .bind(account_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        Ok(rows.into_iter().map(to_transaction).collect())
    }

    async fn insert_p2p_transfer(
        &self,
        sender_account_id: i64,
        receiver_account_id: i64,
        external_id: &str,
        sender_name: &str,
        sender_document: &str,
        receiver_name: &str,
        receiver_document: &str,
        amount: Decimal,
        description: &str,
    ) -> Result<(i64, i64), RepositoryError> {
        let mut db_tx = self.write.begin().await?;
        let (debit_id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT_P2P_DEBIT)
            .bind(sender_account_id)
            .bind(0_i64)
            .bind(external_id)
            .bind(receiver_name)
            .bind(receiver_document)
            .bind(amount)
            .bind(description)
            .fetch_one(&mut *db_tx)
            .await?;
        let (credit_id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT_P2P_CREDIT)
            .bind(receiver_account_id)
            .bind(debit_id)
            .bind(external_id)
            .bind(sender_name)
            .bind(sender_document)
            .bind(amount)
            .bind(description)
            .fetch_one(&mut *db_tx)
            .await?;
        db_tx.commit().await?;
        Ok((debit_id, credit_id))
    }

    async fn get_customer_activities(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<ItemsPage<Vec<CustomerActivityRow>>, RepositoryError> {
        #[derive(sqlx::FromRow)]
        struct ActRow {
            customer_id: i64,
            full_name: Option<String>,
            tx_count: i64,
            last_activity: Option<DateTime<Utc>>,
        }
        let rows: Vec<ActRow> = sqlx::query_as(ddl::SQL_CUSTOMER_ACTIVITIES)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.read.as_ref())
            .await?;
        let total: (i64,) = sqlx::query_as(ddl::SQL_CUSTOMER_ACTIVITIES_COUNT)
            .fetch_one(self.read.as_ref())
            .await?;
        let items: Vec<CustomerActivityRow> = rows
            .into_iter()
            .map(|r| CustomerActivityRow {
                customer_id: r.customer_id,
                full_name: r.full_name,
                tx_count: r.tx_count,
                last_activity: r.last_activity,
            })
            .collect();
        Ok(ItemsPage {
            offset,
            limit,
            total: total.0,
            items,
        })
    }

    async fn insert_pix_in_credit(
        &self,
        account_id: i64,
        invoice_id: i64,
        partners_id: i64,
        external_id: &str,
        name: &str,
        document_number: &str,
        amount: Decimal,
        key: &str,
        remittance_information: &str,
        gateway: &str,
        pix_operation_type: &str,
        requested_amount: Decimal,
        net_amount: Decimal,
        total_amount: Decimal,
        fee_fixed: Decimal,
        fee_percent_rate: Decimal,
        fee_percent_amount: Decimal,
        fee_total: Decimal,
        partner_fixed_cash_in: Decimal,
        partner_percent_cashin: Decimal,
    ) -> Result<i64, RepositoryError> {
        let (id,): (i64,) = sqlx::query_as(ddl::SQL_INSERT_PIX_IN_CREDIT)
            .bind(account_id)
            .bind(invoice_id)
            .bind(partners_id)
            .bind(external_id)
            .bind(name)
            .bind(document_number)
            .bind(amount)
            .bind(key)
            .bind(remittance_information)
            .bind(if gateway.is_empty() { None::<&str> } else { Some(gateway) })
            .bind(if pix_operation_type.is_empty() { None::<&str> } else { Some(pix_operation_type) })
            .bind(requested_amount)
            .bind(net_amount)
            .bind(total_amount)
            .bind(fee_fixed)
            .bind(fee_percent_rate)
            .bind(fee_percent_amount)
            .bind(fee_total)
            .bind(partner_fixed_cash_in)
            .bind(partner_percent_cashin)
            .fetch_one(self.write.as_ref())
            .await?;
        Ok(id)
    }
}
