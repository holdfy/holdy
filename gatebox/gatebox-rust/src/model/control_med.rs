use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMed {
    #[serde(default)]
    pub id: i64,
    pub account_id: i64,
    pub invoice_id: i64,
    pub partners_id: i64,
    pub bank_id: String,
    pub endtoend: String,
    pub details: String,
    pub status_controle_med_id: i64,
    pub amount: Decimal,
    pub data_med: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
