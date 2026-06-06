//! Order API responses.

use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub amount: String,
    pub status: String,
    pub fiat_rail: String,
    pub risk_score: u32,
    pub risk_decision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custody_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_in_tx_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pix_br_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding_instruction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub off_ramp_tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brlx_escrow_transfer_tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soroban_escrow_contract_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soroban_lock_tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soroban_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_code: Option<String>,
}
