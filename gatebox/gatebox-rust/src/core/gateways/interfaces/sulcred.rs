// DTOs for Sulcred provider (from app/modules/core/gateways/interfaces/sulcred/*)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SulcredSendPixKeyRequest {
    pub expiration: i32,
    #[serde(rename = "pixKey")]
    pub pix_key: String,
    pub priority: Option<String>,
    #[serde(rename = "creditorDocument")]
    pub creditor_document: Option<String>,
    pub description: String,
    pub payment: SulcredSendPixPayment,
    #[serde(rename = "internalTransactionId", skip_serializing_if = "Option::is_none")]
    pub internal_transaction_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SulcredSendPixPayment {
    pub currency: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SulcredSendPixKeyResponse {
    #[serde(rename = "endToEndId")]
    pub end_to_end_id: String,
    #[serde(rename = "eventDate")]
    pub event_date: String,
    pub status: String,
    pub id: i64,
    pub payment: SulcredSendPixKeyPaymentResponse,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: SulcredSendPixKeyData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SulcredSendPixKeyPaymentResponse {
    pub currency: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SulcredSendPixKeyData {
    pub id: i64,
    pub refunds: Vec<serde_json::Value>,
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    #[serde(rename = "endToEndId")]
    pub end_to_end_id: String,
    #[serde(rename = "pixKey")]
    pub pix_key: String,
    pub payment: SulcredSendPixKeyDataPayment,
    pub status: String,
    #[serde(rename = "transactionType")]
    pub transaction_type: String,
    #[serde(rename = "localInstrument")]
    pub local_instrument: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "creditorAccount")]
    pub creditor_account: SulcredSendPixKeyAccount,
    #[serde(rename = "debtorAccount")]
    pub debtor_account: SulcredSendPixKeyAccount,
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
pub struct SulcredSendPixKeyDataPayment {
    pub currency: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SulcredSendPixKeyAccount {
    pub ispb: String,
    pub document: String,
    pub name: String,
    pub number: String,
    pub issuer: String,
    #[serde(rename = "accountType")]
    pub account_type: String,
}

// PIX IN - dynamic QR code (cob)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SulcredCreateDynamicBrcodeRequest {
    pub calendario: Option<SulcredCalendario>,
    pub devedor: SulcredDevedor,
    pub valor: SulcredValor,
    pub chave: String,
    pub solicitacao_pagador: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SulcredCalendario {
    pub expiracao: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SulcredDevedor {
    pub cpf: Option<String>,
    pub cnpj: Option<String>,
    pub nome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SulcredValor {
    pub original: String,
    pub modalidade_alteracao: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SulcredCreateDynamicBrcodeResponse {
    #[serde(rename = "pixCopiaECola")]
    pub pix_copia_e_cola: String,
    pub txid: Option<String>,
    pub location: Option<String>,
    pub status: Option<String>,
}
