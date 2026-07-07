use serde::{Deserialize, Serialize};

// --- Payin Quote (PIX IN: fiat → stablecoin) ---
// Campos confirmados pela API real (06/2026):
//   blockchain_wallet_id: exatamente 15 chars (ex: bw_XXXXXXXXXXXX)
//   currency_type: "sender" | "receiver"
//   request_amount: number >= 500 (centavos BRL)
//   payment_method: "pix" | "ted" | "spei" | "ach" | "wire" | ...
//   token: "USDC" | "USDT" | "USDB"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayPayinQuoteRequest {
    pub blockchain_wallet_id: String,
    pub currency_type: String,    // "receiver" para receber na carteira
    pub request_amount: f64,      // em centavos BRL (min 500 = R$ 5,00)
    pub payment_method: String,   // "pix" (lowercase)
    pub token: String,            // "USDB" em dev, "USDC" em prod
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayPayinQuoteResponse {
    pub id: String,
    pub expires_at: Option<i64>,
    pub commercial_quotation: Option<f64>,
    pub blindpay_quotation: Option<f64>,
    pub partner_fee_amount: Option<f64>,
    pub receiver_amount: Option<f64>,
    pub sender_amount: Option<f64>,
    pub flat_fee: Option<f64>,
    pub billing_fee_amount: Option<f64>,
}

// --- Payin (creates PIX QR code) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayPayinRequest {
    pub payin_quote_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayPayinResponse {
    pub id: String,
    pub status: Option<String>,
    pub amount: Option<serde_json::Value>,
    // PIX copia-e-cola — campo pode variar por versão da API
    #[serde(alias = "pix_code", alias = "brcode", alias = "qr_code", alias = "pix_brcode")]
    pub pix_code: Option<String>,
    pub pix_qr_code: Option<String>,
    pub txid: Option<String>,
    // Campos extras comuns na resposta
    pub payment_method: Option<String>,
    pub fiat_amount: Option<f64>,
}

// --- Payout Quote (stablecoin → PIX OUT) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayPayoutQuoteRequest {
    pub customer_id: String,
    pub bank_account_id: String,
    pub request_amount: f64,      // renomeado de amount para alinhar com API
    pub cover_fees: bool,
    pub blockchain: String,       // "stellar_testnet" em dev, "stellar" em prod
    pub token: String,            // "USDB" em dev, "USDC" em prod
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayPayoutQuoteResponse {
    pub id: String,
    pub request_amount: Option<f64>,
    pub fiat_amount: Option<f64>,
    pub fee: Option<serde_json::Value>,
    pub expires_at: Option<String>,
    pub status: Option<String>,
}

// --- Authorize Stellar (get XDR to sign) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayAuthorizeStellarRequest {
    pub quote_id: String,
    pub blockchain_wallet_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayAuthorizeStellarResponse {
    pub xdr: String,
    pub id: Option<String>,
}

// --- Payout (execute after signing XDR) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayPayoutRequest {
    pub quote_id: String,
    pub signed_xdr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayPayoutResponse {
    pub id: String,
    pub status: Option<String>,
    pub fiat_amount: Option<f64>,
    pub end_to_end_id: Option<String>,
}

// --- Wallet Registration (Stellar — is_account_abstraction) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayWalletRegisterRequest {
    pub name: String,
    pub network: String,              // "stellar_testnet" | "stellar"
    pub is_account_abstraction: bool, // true para Stellar (endereço externo)
    pub address: String,              // endereço Stellar público (G...)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindPayWalletResponse {
    pub id: Option<String>,
    pub balance: Option<serde_json::Value>,
    pub available_balance: Option<serde_json::Value>,
    pub address: Option<String>,
    pub network: Option<String>,
}
