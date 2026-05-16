// DTOs for SevenTrust provider (from app/modules/core/gateways/interfaces/seventrust/*)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenTrustSendPixKeyRequest {
    pub expiration: i32,
    #[serde(rename = "pixKey")]
    pub pix_key: String,
    pub priority: Option<String>,
    #[serde(rename = "creditorDocument")]
    pub creditor_document: Option<String>,
    pub description: String,
    pub payment: SevenTrustPayment,
    #[serde(rename = "internalTransactionId", skip_serializing_if = "Option::is_none")]
    pub internal_transaction_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenTrustPayment {
    pub currency: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenTrustSendPixKeyResponse {
    #[serde(rename = "endToEndId")]
    pub end_to_end_id: String,
    #[serde(rename = "eventDate")]
    pub event_date: String,
    pub status: String,
    pub id: serde_json::Value,
    pub payment: SevenTrustPaymentResponse,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: SevenTrustPixKeyData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenTrustPaymentResponse {
    pub currency: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenTrustPixKeyData {
    pub id: serde_json::Value,
    pub refunds: Vec<serde_json::Value>,
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    #[serde(rename = "endToEndId")]
    pub end_to_end_id: String,
    #[serde(rename = "pixKey")]
    pub pix_key: String,
    pub payment: SevenTrustPayment,
    pub status: String,
    #[serde(rename = "transactionType")]
    pub transaction_type: String,
    #[serde(rename = "localInstrument")]
    pub local_instrument: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "creditorAccount")]
    pub creditor_account: SevenTrustAccount,
    #[serde(rename = "debtorAccount")]
    pub debtor_account: SevenTrustAccount,
    #[serde(rename = "remittanceInformation")]
    pub remittance_information: String,
    #[serde(rename = "errorCode")]
    pub error_code: String,
    #[serde(rename = "txId")]
    pub tx_id: String,
    #[serde(rename = "creditDebitType")]
    pub credit_debit_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenTrustAccount {
    pub ispb: String,
    pub issuer: String,
    pub number: String,
    #[serde(rename = "accountType")]
    pub account_type: String,
    pub document: String,
    pub name: String,
}
