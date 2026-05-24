// Next (NextPay) HTTP service - real implementation with reqwest
use async_trait::async_trait;
use reqwest::Client;

use crate::core::gateways::interfaces::next::{
    CustomerSendPix, DestinatarySendPix, NextSendPixRequest, NextSendPixResponse,
};
use crate::core::gateways::interfaces::sulcred::SulcredSendPixKeyRequest;
use crate::core::gateways::services::traits::{
    AuthOutResponse, BalanceResponse, CreateDynamicQrcodeRequest, CreateDynamicQrcodeResponse,
    GatewayHttpService, SendPixKeyResponse,
};
use crate::core::rabbitmq::GatewayFailureConfig;

fn next_auth_url() -> String {
    std::env::var("NEXTAUTH").unwrap_or_else(|_| "https://auth.nextpay.example".to_string())
}

fn next_api_url() -> String {
    std::env::var("NEXTAPI").unwrap_or_else(|_| "https://api.nextpay.example".to_string())
}

/// Next (NextPay) HTTP service - token from NEXTAUTH, PIX from NEXTAPI.
pub struct NextHttpService {
    client: Client,
    auth_url: String,
    api_url: String,
}

impl NextHttpService {
    pub fn new() -> Self {
        Self::new_with_urls(next_auth_url(), next_api_url())
    }

    pub fn new_with_urls(auth_url: String, api_url: String) -> Self {
        NextHttpService {
            client: Client::new(),
            auth_url,
            api_url,
        }
    }
}

impl Default for NextHttpService {
    fn default() -> Self {
        Self::new()
    }
}

fn infer_key_type(key: &str) -> &'static str {
    if key.contains('@') {
        "EMAIL"
    } else if key.len() == 11 && key.chars().all(|c| c.is_ascii_digit()) {
        "PHONE"
    } else if key.len() == 14 && key.chars().all(|c| c.is_ascii_digit()) {
        "CNPJ"
    } else if key.len() <= 14 && key.chars().all(|c| c.is_ascii_digit()) {
        "CPF"
    } else {
        "RANDOM"
    }
}

fn infer_person_type(document: Option<&str>) -> &'static str {
    match document {
        Some(doc) => {
            let digits: String = doc.chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() == 14 {
                "LEGAL_PERSON"
            } else {
                "NATURAL_PERSON"
            }
        }
        None => "NATURAL_PERSON",
    }
}

#[async_trait]
impl GatewayHttpService for NextHttpService {
    async fn get_token_out(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<AuthOutResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/oauth/token", self.auth_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "grant_type": "client_credentials"
        });
        let res = self.client.post(&url).json(&body).send().await?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {}: {}", status, text).into());
        }
        let raw: serde_json::Value = res.json().await?;
        Ok(AuthOutResponse {
            access_token: raw["access_token"].as_str().unwrap_or("").to_string(),
            expires_in: raw["expires_in"].as_i64().unwrap_or(3600) as i32,
            token_type: raw["token_type"].as_str().map(String::from),
        })
    }

    async fn send_pix_key(
        &self,
        token: &str,
        request: &SulcredSendPixKeyRequest,
        failure_config: Option<&GatewayFailureConfig>,
    ) -> Result<SendPixKeyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/pix/send", self.api_url.trim_end_matches('/'));
        let key_type = infer_key_type(&request.pix_key);
        let next_req = NextSendPixRequest {
            customer: CustomerSendPix {
                name: request.creditor_document.as_ref().map(|_| "Sender".to_string()).unwrap_or_default(),
                document: request.creditor_document.clone().unwrap_or_default(),
                email: String::new(),
                mobile_phone: String::new(),
            },
            destinatary: DestinatarySendPix {
                end_to_end_id: format!("E{:014}", chrono::Utc::now().timestamp_millis() % 10_i64.pow(14)),
                person_type: infer_person_type(request.creditor_document.as_deref()).to_string(),
                dict_key: request.pix_key.clone(),
                key_type: key_type.to_string(),
                ispb: 0,
                agency: String::new(),
                account: String::new(),
                date_create_key: String::new(),
                bank_name: String::new(),
            },
            custom: request.internal_transaction_id.clone().unwrap_or_default(),
            remittance_information: request.description.clone(),
            amount: request.payment.amount,
            description: request.description.clone(),
        };
        let mut req = self.client.post(&url).bearer_auth(token).json(&next_req);
        if let Some(cfg) = failure_config {
            if let Ok(json) = serde_json::to_string(cfg) {
                req = req.header("X-Gateway-Failure-Config", json);
            }
        }
        let res = req.send().await?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {}: {}", status, text).into());
        }
        let next_resp: NextSendPixResponse = res.json().await?;
        let end_to_end_id = next_resp.end_to_end_id.clone();
        let status_string = next_resp.status_string.clone();
        let created_at = next_resp.created_at.clone();
        use crate::core::gateways::interfaces::sulcred::{
            SulcredSendPixKeyAccount, SulcredSendPixKeyData, SulcredSendPixKeyDataPayment,
            SulcredSendPixKeyPaymentResponse, SulcredSendPixKeyResponse,
        };
        Ok(SulcredSendPixKeyResponse {
            end_to_end_id: end_to_end_id.clone(),
            event_date: created_at.clone(),
            status: status_string.clone(),
            id: next_resp.id.parse().unwrap_or(0),
            payment: SulcredSendPixKeyPaymentResponse {
                currency: "BRL".to_string(),
                amount: next_resp.amount.to_string(),
            },
            type_: "PIX".to_string(),
            data: SulcredSendPixKeyData {
                id: next_resp.id.parse().unwrap_or(0),
                refunds: vec![],
                idempotency_key: next_resp.custom.clone(),
                end_to_end_id,
                pix_key: next_resp.destinatary.dict_key.clone(),
                payment: SulcredSendPixKeyDataPayment {
                    currency: "BRL".to_string(),
                    amount: next_resp.amount,
                },
                status: status_string,
                transaction_type: "PIX".to_string(),
                local_instrument: "DICT".to_string(),
                created_at,
                creditor_account: SulcredSendPixKeyAccount {
                    ispb: String::new(),
                    document: String::new(),
                    name: String::new(),
                    number: String::new(),
                    issuer: String::new(),
                    account_type: String::new(),
                },
                debtor_account: SulcredSendPixKeyAccount {
                    ispb: String::new(),
                    document: String::new(),
                    name: String::new(),
                    number: String::new(),
                    issuer: String::new(),
                    account_type: String::new(),
                },
                remittance_information: next_resp.remittance_information,
                error_code: String::new(),
                tx_id: next_resp.id.clone(),
                credit_debit_type: "DEBIT".to_string(),
            },
        })
    }

    async fn get_balance(&self, token: &str) -> Result<BalanceResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/accounts/balance", self.api_url.trim_end_matches('/'));
        let res = self.client.get(&url).bearer_auth(token).send().await?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {}: {}", status, text).into());
        }
        let raw: serde_json::Value = res.json().await?;
        let balance = raw.get("balance").and_then(|v| v.as_f64());
        let available = raw.get("available").and_then(|v| v.as_f64()).or(balance);
        Ok(BalanceResponse { balance, available })
    }

    async fn create_dynamic_qrcode(
        &self,
        _token: &str,
        _request: &CreateDynamicQrcodeRequest,
    ) -> Result<CreateDynamicQrcodeResponse, Box<dyn std::error::Error + Send + Sync>> {
        Err("NextHttpService: create_dynamic_qrcode not implemented".into())
    }
}
