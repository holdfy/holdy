use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub id: i64,
    pub postal_code: String,
    pub street: String,
    pub number: String,
    pub address_complement: String,
    pub neighborhood: String,
    pub city: String,
    pub state: String,
    pub address_type_id: i64,
    pub customer_id: i64,
    pub business_id: i64,
    pub deleted_at: Option<DateTime<Utc>>,
    pub company_id: i64,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
