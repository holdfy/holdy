use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPixCache {
    #[serde(default)]
    pub id: i64,
    pub key: String,
    pub pix_key_type_id: i64,
    pub document_number: String,
    pub description: String,
    pub bank_name: String,
    pub account_number: String,
    pub branch: String,
    pub ispb: String,
    pub hide_document: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
