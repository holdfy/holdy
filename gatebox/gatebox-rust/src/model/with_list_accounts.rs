use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithListAccounts {
    #[serde(default)]
    pub id: i64,
    pub type_external_id: i64,
    pub account_id: i64,
    pub document: String,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
