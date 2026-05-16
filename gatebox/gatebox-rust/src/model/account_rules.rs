use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRules {
    #[serde(default)]
    pub id: i64,
    pub account_id: i64,
    pub receive_external: bool,
    pub deposit_external: bool,
    pub descricao: String,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
