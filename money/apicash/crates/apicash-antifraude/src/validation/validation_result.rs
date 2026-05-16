//! Shared DTOs produced by validators (SEFAZ / social).

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// High-level status for an individual taxpayer document (CPF-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SefazPersonStatus {
    /// Cleared as *regular* with Receita Federal (mock or live API).
    Regular,
    /// Not regular, suspended, or flagged.
    Irregular,
    /// Could not determine (timeout, rate limit, parsing error).
    Unknown,
}

/// Outcome of a CPF/CNPJ consult (structure ready for FiscalAPI-style JSON mapping).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SefazValidationResult {
    pub normalized_document: String,
    pub status: SefazPersonStatus,
    /// Raw provider payload for audit (redact in production logs).
    pub provider_hint: Option<JsonValue>,
}

/// Parsed social profile used for scoring (age / handle consistency).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAccountSnapshot {
    pub platform: String,
    pub handle: String,
    /// Best-effort estimate of account age in months (from API or mock).
    pub estimated_age_months: u32,
    /// Whether display name loosely matches KYC name (mock heuristic).
    pub name_consistent: bool,
}

/// One URL validation outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialValidationResult {
    pub url: String,
    pub snapshot: Option<SocialAccountSnapshot>,
    pub error: Option<String>,
}
