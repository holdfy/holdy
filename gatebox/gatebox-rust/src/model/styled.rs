use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Styled {
    #[serde(default)]
    pub id: i64,
    pub url: String,
    pub application_name: String,
    pub title: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub font_color: String,
    pub img: String,
    pub favicon: String,
    pub styled_type_id: i64,
    pub company_id: i64,
    pub active: bool,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
