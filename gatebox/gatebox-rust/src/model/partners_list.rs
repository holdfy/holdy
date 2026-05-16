use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnersList {
    pub id: i64,
    pub description: String,
    pub site: String,
    pub contact: String,
    pub active: bool,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
