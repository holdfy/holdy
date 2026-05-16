use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accounts {
    pub id: i64,
    pub account_number: String,
    pub branch: String,
    pub account_type_id: i64,
    pub account_status_id: i64,
    pub deleted_at: Option<DateTime<Utc>>,
    pub authentication_id: i64,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
