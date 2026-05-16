use chrono::{DateTime, Utc};
use serde::Serialize;

/// One row from transaction_anchors (audit list).
#[derive(Debug, Clone, Serialize)]
pub struct TransactionAnchorRow {
    pub id: i64,
    pub idempotency_key: String,
    pub entity_type: String,
    pub entity_id: String,
    pub payload_hash: String,
    pub period_type: Option<String>,
    pub period_id: Option<String>,
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
    pub chain_id: Option<i64>,
    pub anchored_at: Option<DateTime<Utc>>,
    pub dry_run: bool,
    pub error_message: Option<String>,
    pub account_id: i64,
    pub created_at: DateTime<Utc>,
}

/// Response item for GET /audit (includes explorer_url).
#[derive(Debug, Clone, Serialize)]
pub struct AuditItem {
    pub id: i64,
    pub idempotency_key: String,
    pub entity_type: String,
    pub entity_id: String,
    pub payload_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchored_at: Option<DateTime<Utc>>,
    pub dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explorer_url: Option<String>,
    pub created_at: DateTime<Utc>,
}
