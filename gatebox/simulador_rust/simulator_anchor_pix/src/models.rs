use serde::{Deserialize, Serialize};

// ─── Requests ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    #[allow(dead_code)]
    pub asset: Option<String>,
    pub amount: String,
    pub memo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    #[allow(dead_code)]
    pub asset: Option<String>,
    pub amount: String,
    pub pix_key: String,
}

// ─── Responses ────────────────────────────────────────────────────────────────

/// Compatível com o que `AnchorClient::request_deposit_pix` espera (campos lidos via serde_json::Value).
#[derive(Debug, Serialize)]
pub struct DepositResponse {
    pub transaction_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pix_br_code: Option<String>,
    /// Hash mock — APICash sobrescreve com hash Stellar real após transfer_brlx_to_escrow.
    pub stellar_tx_hash: String,
    pub estimated_completion: String,
    pub gateway: String,
    pub asset: String,
    pub amount: String,
    pub simulated: bool,
}

/// Compatível com `AnchorClient::get_pix_transaction` (lido via `.get("status")`).
#[derive(Debug, Serialize)]
pub struct TransactionStatusResponse {
    pub id: String,
    pub status: String,
    pub amount: String,
    pub asset: String,
}

/// Compatível com `AnchorClient::request_withdraw_pix` (campos lidos via serde_json::Value).
#[derive(Debug, Serialize)]
pub struct WithdrawResponse {
    pub transaction_id: String,
    pub tx_hash: String,
    pub status: String,
    /// Decimal como string para o AnchorClient parsear: `v.get("received_pix").as_str().parse()`.
    pub received_pix: String,
    pub gateway: String,
    pub pix_key: String,
    pub simulated: bool,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}
