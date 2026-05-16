use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsPageModel {
    pub offset: i64,
    pub limit: i64,
    pub total: i64,
    pub items: serde_json::Value,
}
