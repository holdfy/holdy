use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedKey {
    #[serde(default)]
    pub id: i64,
    pub key: String,
    pub pix_key_type_id: i64,
    pub description: String,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
