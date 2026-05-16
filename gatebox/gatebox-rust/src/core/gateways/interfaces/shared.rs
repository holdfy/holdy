// DTOs shared across gateways (from app/modules/core/gateways/interfaces/shared/*)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyOutWebHook {
    pub event: String,
    pub invoice: Option<NotifyInvoice>,
    pub transaction: NotifyTransaction,
    #[serde(rename = "bankData")]
    pub bank_data: NotifyBankData,
    #[serde(rename = "accountId")]
    pub account_id: i32,
    pub status: Option<String>,
    pub notify: Option<bool>,
    #[serde(rename = "notifyTimer")]
    pub notify_timer: Option<String>,
    #[serde(rename = "tryNotify")]
    pub try_notify: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyInvoice {
    pub uuid: String,
    pub emv: String,
    #[serde(rename = "externalId")]
    pub external_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyTransaction {
    pub uuid: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "externalId")]
    pub external_id: Option<String>,
    pub amount: Option<f64>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[serde(rename = "subType")]
    pub sub_type: Option<String>,
    pub ispb: Option<String>,
    pub account: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyBankData {
    pub key: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "documentNumber")]
    pub document_number: Option<String>,
    #[serde(rename = "endtoendId")]
    pub endtoend_id: Option<String>,
    #[serde(rename = "originalendtoendId")]
    pub originalendtoend_id: Option<String>,
    pub identifier: Option<String>,
    pub ispb: Option<String>,
    pub account: Option<String>,
    pub txid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLoggedManager {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub username: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub active: bool,
    #[serde(rename = "forceReset")]
    pub force_reset: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updateAt")]
    pub update_at: String,
    #[serde(rename = "deleteAt")]
    pub delete_at: String,
    #[serde(rename = "Management")]
    pub management: Vec<ManagementItem>,
    #[serde(rename = "Accounts")]
    pub accounts: Vec<AccountItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementItem {
    pub id: i32,
    pub uuid: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    #[serde(rename = "socialName")]
    pub social_name: String,
    #[serde(rename = "typePerson")]
    pub type_person: String,
    #[serde(rename = "documentNumber")]
    pub document_number: String,
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    pub email: String,
    #[serde(rename = "telegramChatId")]
    pub telegram_chat_id: String,
    pub status: String,
    #[serde(rename = "isPoliticallyExposedPerson")]
    pub is_politically_exposed_person: bool,
    #[serde(rename = "authenticationId")]
    pub authentication_id: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updateAt")]
    pub update_at: String,
    #[serde(rename = "deleteAt")]
    pub delete_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountItem {
    pub id: i32,
    pub uuid: String,
    #[serde(rename = "uuidForeigner")]
    pub uuid_foreigner: String,
    #[serde(rename = "accountNumber")]
    pub account_number: String,
    pub branch: String,
    #[serde(rename = "accountOnboardingType")]
    pub account_onboarding_type: String,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updateAt")]
    pub update_at: String,
    #[serde(rename = "deleteAt")]
    pub delete_at: String,
    #[serde(rename = "authenticationId")]
    pub authentication_id: i32,
}
