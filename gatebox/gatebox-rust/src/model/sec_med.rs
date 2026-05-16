use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecMed {
    #[serde(default)]
    pub id: i64,
    pub account_id: i64,
    pub invoice_id: i64,
    pub partners_id: i64,
    pub apagar: String,
    pub transaction_id: i64,
    pub status_sec_med_id: i64,
    pub amount: Decimal,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
