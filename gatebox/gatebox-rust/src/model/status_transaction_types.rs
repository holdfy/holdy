use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusTransactionTypes {
    pub id: i64,
    pub code: String,
    pub description: String,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
