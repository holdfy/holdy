//! On-ramp: fiat (PIX) → token (e.g. BRLx on Stellar).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Response after initiating a PIX-funded deposit through the Anchor HTTP API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnRampResponse {
    /// Internal provider transaction identifier (anchor tx id).
    #[serde(default)]
    pub transaction_id: Option<String>,
    /// Correlation key chosen by APICash (idempotency/reference).
    #[serde(default)]
    pub external_id: Option<String>,
    /// Active fiat rail (`anchor`).
    pub fiat_rail: String,
    pub stellar_tx_hash: String,
    pub status: String,
    /// PIX copia-e-cola payload (BR Code).
    #[serde(default)]
    pub pix_br_code: Option<String>,
    #[serde(default)]
    pub gateway: Option<String>,
    pub estimated_completion: DateTime<Utc>,
}
