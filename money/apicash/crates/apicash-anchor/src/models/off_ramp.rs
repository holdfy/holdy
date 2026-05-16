//! Off-ramp: token → fiat payout (PIX).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Response after requesting withdrawal to a PIX key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffRampResponse {
    /// Internal provider transaction identifier.
    #[serde(default)]
    pub transaction_id: Option<String>,
    /// Correlation key chosen by APICash (idempotency/reference).
    #[serde(default)]
    pub external_id: Option<String>,
    pub tx_hash: String,
    pub status: String,
    #[serde(default)]
    pub gateway: Option<String>,
    /// Fiat amount expected to be received (same currency as Anchor settlement).
    pub received_pix: Decimal,
}
