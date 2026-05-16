use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    #[serde(default)]
    pub id: i64,
    pub identifier: String,
    pub key: String,
    pub pix_key_type_id: i64,
    pub invoice_type_id: i64,
    pub timeout: i64,
    pub expire: i64,
    pub partners_list_id: i64,
    pub amount: Decimal,
    pub invoice_status_id: i64,
    pub external_id: String,
    pub document_number: String,
    pub description: String,
    pub account_id: i64,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
