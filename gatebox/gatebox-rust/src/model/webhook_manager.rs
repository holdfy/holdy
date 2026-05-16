use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookManager {
    #[serde(default)]
    pub id: i64,
    pub callback_url: String,
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub webhook_type_id: i64,
    pub account_id: i64,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
