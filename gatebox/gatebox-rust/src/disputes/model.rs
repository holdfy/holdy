use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub id: i64,
    pub transaction_id: Option<i64>,
    pub account_id: i64,
    /// INFRACTION | REVERSAL | FRAUD
    pub r#type: String,
    /// OPEN | RESOLVED | CLOSED
    pub status: String,
    pub reason: String,
    pub evidence: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<i64>,
    pub resolution_notes: Option<String>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDisputeRequest {
    pub transaction_id: Option<i64>,
    pub account_id: i64,
    pub r#type: Option<String>,
    pub reason: String,
    pub evidence: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveDisputeRequest {
    /// "customer" | "platform"
    pub resolution: String,
    pub notes: Option<String>,
    pub resolved_by: Option<i64>,
}
