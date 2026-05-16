//! Normalized Stellar transaction view (Horizon or internal tracking).

use serde::{Deserialize, Serialize};

/// Transaction envelope for status polling and UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarTransaction {
    pub id: String,
    pub amount: String,
    pub asset: String,
    pub from: String,
    pub to: String,
    pub memo: String,
}
