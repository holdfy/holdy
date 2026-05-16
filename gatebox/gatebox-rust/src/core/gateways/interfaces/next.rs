// DTOs for Next provider (from app/modules/core/gateways/interfaces/next/*)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextSendPixRequest {
    pub customer: CustomerSendPix,
    pub destinatary: DestinatarySendPix,
    pub custom: String,
    #[serde(rename = "remittanceInformation")]
    pub remittance_information: String,
    pub amount: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSendPix {
    pub name: String,
    pub document: String,
    pub email: String,
    #[serde(rename = "mobilePhone")]
    pub mobile_phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinatarySendPix {
    #[serde(rename = "endToEndId")]
    pub end_to_end_id: String,
    #[serde(rename = "personType")]
    pub person_type: String,
    #[serde(rename = "dictKey")]
    pub dict_key: String,
    #[serde(rename = "keyType")]
    pub key_type: String,
    pub ispb: i32,
    pub agency: String,
    pub account: String,
    #[serde(rename = "dateCreateKey")]
    pub date_create_key: String,
    #[serde(rename = "bankName")]
    pub bank_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextSendPixResponse {
    pub id: String,
    pub custom: String,
    #[serde(rename = "principalAmount")]
    pub principal_amount: f64,
    pub amount: f64,
    pub description: String,
    pub fee: f64,
    #[serde(rename = "serviceFee")]
    pub service_fee: f64,
    #[serde(rename = "feeAmount")]
    pub fee_amount: f64,
    #[serde(rename = "remittanceInformation")]
    pub remittance_information: String,
    #[serde(rename = "statusstring")]
    pub status_string: String,
    #[serde(rename = "statusCode")]
    pub status_code: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "endToEndId")]
    pub end_to_end_id: String,
    pub success: bool,
    pub destinatary: DestinatarySendPixResponse,
    pub payer: PayerSendPixResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinatarySendPixResponse {
    #[serde(rename = "dictKey")]
    pub dict_key: String,
    pub name: String,
    pub document: String,
    pub email: String,
    #[serde(rename = "mobilePhone")]
    pub mobile_phone: String,
    #[serde(rename = "bankAccount")]
    pub bank_account: DestinataryBankAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinataryBankAccount {
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayerSendPixResponse {
    #[serde(rename = "holderName")]
    pub holder_name: String,
    #[serde(rename = "holderDocument")]
    pub holder_document: String,
    #[serde(rename = "bankAccount")]
    pub bank_account: BankAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for DestinataryBankAccount {
    fn default() -> Self {
        Self { extra: std::collections::HashMap::new() }
    }
}
impl Default for BankAccount {
    fn default() -> Self {
        Self { extra: std::collections::HashMap::new() }
    }
}
