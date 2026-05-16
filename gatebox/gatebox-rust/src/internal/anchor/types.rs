use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    PixTx,
    Med,
    BalanceSnapshot,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::PixTx => "pix_tx",
            EntityType::Med => "med",
            EntityType::BalanceSnapshot => "balance_snapshot",
        }
    }

    pub fn valid(s: &str) -> bool {
        matches!(s, "pix_tx" | "med" | "balance_snapshot")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeriodType {
    Day,
    Week,
    Fortnight,
    Month,
    Year,
}

impl PeriodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PeriodType::Day => "day",
            PeriodType::Week => "week",
            PeriodType::Fortnight => "fortnight",
            PeriodType::Month => "month",
            PeriodType::Year => "year",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPayload {
    pub schema_version: String,
    pub idempotency_key: String,
    pub entity_type: String,
    pub entity_id: String,
    pub payload_hash: String,
    pub occurred_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    pub account_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_document: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}
