use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenService {
    #[serde(default)]
    pub id: i64,
    pub description: String,
    pub token: String,
    pub expire_in: String,
    pub authentication_id: i64,
    pub timestamp: Option<DateTime<Utc>>,
    pub active: bool,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
