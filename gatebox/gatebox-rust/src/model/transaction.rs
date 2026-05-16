use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(default)]
    pub id: i64,
    pub account_id: i64,
    pub invoice_id: i64,
    pub partners_id: i64,
    pub transaction_id: String,
    pub charger_back_id: String,
    pub parent_id: i64,
    pub external_id: String,
    pub name: String,
    pub email: String,
    pub document_number: String,
    pub description: String,
    pub phone: String,
    pub amount: Decimal,
    pub isbp: String,
    pub bank_name: String,
    pub branch: String,
    pub account: String,
    pub endtoend_id: String,
    pub pix_key_type_id: i64,
    pub key: String,
    pub type_transaction_id: i64,
    pub sub_type_transaction_id: i64,
    pub remittance_information: String,
    pub status_transaction_id: i64,
    pub msg_error: String,
    pub telegram_notification: bool,
    pub try_count: i64,
    pub deleted_at: Option<DateTime<Utc>>,
    pub endtoend_id_temp: String,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
