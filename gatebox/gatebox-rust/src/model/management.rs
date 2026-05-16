use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Management {
    pub id: i64,
    pub full_name: String,
    pub social_name: String,
    pub type_person_id: i64,
    pub document_number: String,
    pub phone_number: String,
    pub email: String,
    pub telegram_chat_id: String,
    pub customer_status_id: i64,
    pub is_politically_exposed_person: bool,
    pub authentication_id: i64,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
