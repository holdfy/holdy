//! DTOs produced by the social validator.

use serde::{Deserialize, Serialize};

/// Parsed social profile used for scoring (age / handle consistency).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAccountSnapshot {
    pub platform: String,
    pub handle: String,
    /// Best-effort estimate of account age in months (from API or mock heuristic).
    pub estimated_age_months: u32,
    /// Whether display name loosely matches KYC name.
    pub name_consistent: bool,
}

/// Outcome of validating a single social profile URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialValidationResult {
    pub url: String,
    pub snapshot: Option<SocialAccountSnapshot>,
    pub error: Option<String>,
}
