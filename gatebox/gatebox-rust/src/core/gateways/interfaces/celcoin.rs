// DTOs for Celcoin provider (from app/modules/core/gateways/interfaces/celcoin/*)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelcoinPixOutRequest {
    #[serde(rename = "accountNumber")]
    pub account_number: String,
    #[serde(rename = "transactionIdentification")]
    pub transaction_identification: String,
    pub amount: f64,
    pub key: String,
    #[serde(rename = "keyType")]
    pub key_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelcoinPixOutResponse {
    pub status: String,
    pub version: String,
    pub body: CelcoinPixOutBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelcoinPixOutBody {
    pub id: String,
    pub amount: f64,
    #[serde(rename = "clientCode")]
    pub client_code: String,
    #[serde(rename = "transactionIdentification")]
    pub transaction_identification: Option<String>,
    #[serde(rename = "endToEndId")]
    pub end_to_end_id: String,
    #[serde(rename = "initiationType")]
    pub initiation_type: String,
    #[serde(rename = "paymentType")]
    pub payment_type: String,
    pub urgency: String,
    #[serde(rename = "transactionType")]
    pub transaction_type: String,
    #[serde(rename = "debitParty")]
    pub debit_party: CelcoinDebitPartyOut,
    #[serde(rename = "creditParty")]
    pub credit_party: CelcoinCreditPartyOut,
    #[serde(rename = "remittanceInformation")]
    pub remittance_information: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelcoinDebitPartyOut {
    pub account: String,
    pub branch: String,
    #[serde(rename = "taxId")]
    pub tax_id: String,
    pub name: String,
    #[serde(rename = "accountType")]
    pub account_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelcoinCreditPartyOut {
    pub bank: String,
    pub key: Option<String>,
    pub account: String,
    pub branch: String,
    #[serde(rename = "taxId")]
    pub tax_id: String,
    pub name: String,
    #[serde(rename = "accountType")]
    pub account_type: String,
}
