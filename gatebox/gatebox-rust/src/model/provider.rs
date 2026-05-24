use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub admin: AdminProviderInfo,
    pub customer: CustomerProviderInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminProviderInfo {
    pub fixed_cash_in: f64,
    pub fixed_cash_out: f64,
    pub percent_cashin: f64,
    pub percent_cashout: f64,
    pub fixed_ref_cashin: f64,
    pub fixed_ref_cashout: f64,
    pub percent_ref_cashin: f64,
    pub percent_ref_cashout: f64,
    pub data: AdminData,
    #[serde(rename = "accountId")]
    pub account_id: i64,
    pub partners: PartnersInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerProviderInfo {
    pub data: CustomerData,
    #[serde(rename = "accountId")]
    pub account_id: i64,
    /// 1 = NATURAL_PERSON (PF), 2 = LEGAL_PERSON (PJ), 0 = unknown.
    pub person_type_id: i64,
    pub fixed_cash_in: f64,
    pub fixed_cash_out: f64,
    pub percent_cashin: f64,
    pub percent_cashout: f64,
    pub percent_sec_med: f64,
    pub fixed_ref_cash_in: f64,
    pub fixed_ref_cashout: f64,
    pub percent_ref_cashin: f64,
    pub percent_ref_cashout: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnersInfo {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    pub fixed_cash_in: f64,
    pub fixed_cash_out: f64,
    pub percent_cashin: f64,
    pub percent_cashout: f64,
    pub fixed_ref_cash_in: f64,
    pub fixed_ref_cash_out: f64,
    pub percent_ref_cashin: f64,
    pub percent_ref_cashout: f64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminData {
    #[serde(rename = "authenticationId")]
    pub authentication_id: i64,
    #[serde(rename = "fullName")]
    pub full_name: String,
    #[serde(rename = "documentNumber")]
    pub document_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerData {
    #[serde(rename = "authenticationId")]
    pub authentication_id: i64,
    #[serde(rename = "fullName")]
    pub full_name: String,
    #[serde(rename = "documentNumber")]
    pub document_number: String,
    pub email: String,
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
}
