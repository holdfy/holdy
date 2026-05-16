use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authentication {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub password: String,
    pub type_auth_id: i64,
    pub active: bool,
    pub force_reset: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
