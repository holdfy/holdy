use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ItemsPage<T> {
    pub offset: i64,
    pub limit: i64,
    pub total: i64,
    pub items: T,
}
