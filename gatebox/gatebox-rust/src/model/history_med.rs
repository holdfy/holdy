use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMed {
    #[serde(default)]
    pub id: i64,
    pub account_id: i64,
    pub control_med_id: i64,
    pub sec_med_id: i64,
    pub apagar: String,
    pub amount: Decimal,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
