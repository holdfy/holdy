// SevenTrust HTTP service (from app/modules/core/gateways/services/seventrust)
use async_trait::async_trait;
use reqwest::Client;

use crate::core::gateways::interfaces::seventrust::{
    SevenTrustPayment, SevenTrustSendPixKeyRequest as StSendPixReq, SevenTrustSendPixKeyResponse as StSendPixResp,
};
use crate::core::gateways::interfaces::sulcred::{
    SulcredSendPixKeyAccount, SulcredSendPixKeyData, SulcredSendPixKeyDataPayment,
    SulcredSendPixKeyPaymentResponse, SulcredSendPixKeyResponse,
};
use crate::core::gateways::services::traits::{
    AuthOutResponse, BalanceResponse, CreateDynamicQrcodeRequest, CreateDynamicQrcodeResponse,
    GatewayHttpService, SendPixKeyResponse,
};
use crate::core::rabbitmq::GatewayFailureConfig;

fn base_url_out() -> String {
    std::env::var("SEVENOUT_URL").unwrap_or_else(|_| "http://localhost:7010".to_string())
}

/// SevenTrust HTTP service - real implementation (token, send PIX, balance).
pub struct SevenTrustHttpService {
    client: Client,
    base_url_out: String,
}

impl Default for SevenTrustHttpService {
    fn default() -> Self {
        Self::new(Client::new(), base_url_out())
    }
}

impl SevenTrustHttpService {
    pub fn new(client: Client, base_url_out: String) -> Self {
        SevenTrustHttpService { client, base_url_out }
    }
}

#[async_trait]
impl GatewayHttpService for SevenTrustHttpService {
    async fn get_token_out(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<AuthOutResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v2/auth/microservice/out", self.base_url_out.trim_end_matches('/'));
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
        request: &crate::core::gateways::interfaces::sulcred::SulcredSendPixKeyRequest,
        failure_config: Option<&GatewayFailureConfig>,
    ) -> Result<SendPixKeyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v2/pix/send/key", self.base_url_out.trim_end_matches('/'));
        let body = StSendPixReq {
            expiration: request.expiration,
            pix_key: request.pix_key.clone(),
            priority: request.priority.clone(),
            creditor_document: request.creditor_document.clone(),
            description: request.description.clone(),
            payment: SevenTrustPayment {
                currency: request.payment.currency.clone(),
                amount: request.payment.amount,
            },
            internal_transaction_id: request.internal_transaction_id.clone(),
        };
        let mut req = self.client.post(&url).bearer_auth(token).json(&body);
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
        let st: StSendPixResp = res.json().await?;
        let out = SulcredSendPixKeyResponse {
            end_to_end_id: st.end_to_end_id,
            event_date: st.event_date,
            status: st.status,
            id: st.id.as_i64().unwrap_or(0),
            payment: SulcredSendPixKeyPaymentResponse {
                currency: st.payment.currency,
                amount: st.payment.amount,
            },
            type_: st.type_,
            data: SulcredSendPixKeyData {
                id: st.data.id.as_i64().unwrap_or(0),
                refunds: st.data.refunds,
                idempotency_key: st.data.idempotency_key,
                end_to_end_id: st.data.end_to_end_id,
                pix_key: st.data.pix_key,
                payment: SulcredSendPixKeyDataPayment {
                    currency: st.data.payment.currency,
                    amount: st.data.payment.amount,
                },
                status: st.data.status,
                transaction_type: st.data.transaction_type,
                local_instrument: st.data.local_instrument,
                created_at: st.data.created_at,
                creditor_account: SulcredSendPixKeyAccount {
                    ispb: st.data.creditor_account.ispb.clone(),
                    document: st.data.creditor_account.document.clone(),
                    name: st.data.creditor_account.name.clone(),
                    number: st.data.creditor_account.number.clone(),
                    issuer: st.data.creditor_account.issuer.clone(),
                    account_type: st.data.creditor_account.account_type.clone(),
                },
                debtor_account: SulcredSendPixKeyAccount {
                    ispb: st.data.debtor_account.ispb.clone(),
                    document: st.data.debtor_account.document.clone(),
                    name: st.data.debtor_account.name.clone(),
                    number: st.data.debtor_account.number.clone(),
                    issuer: st.data.debtor_account.issuer.clone(),
                    account_type: st.data.debtor_account.account_type.clone(),
                },
                remittance_information: st.data.remittance_information,
                error_code: st.data.error_code,
                tx_id: st.data.tx_id,
                credit_debit_type: st.data.credit_debit_type,
            },
        };
        Ok(out)
    }

    async fn get_balance(&self, token: &str) -> Result<BalanceResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v2/accounts/balances", self.base_url_out.trim_end_matches('/'));
        let res = self.client.get(&url).bearer_auth(token).send().await?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {}: {}", status, text).into());
        }
        let raw: serde_json::Value = res.json().await?;
        Ok(BalanceResponse {
            balance: raw.get("balance").and_then(|v| v.as_f64()),
            available: raw.get("available").and_then(|v| v.as_f64()),
        })
    }

    async fn create_dynamic_qrcode(
        &self,
        _token: &str,
        _request: &CreateDynamicQrcodeRequest,
    ) -> Result<CreateDynamicQrcodeResponse, Box<dyn std::error::Error + Send + Sync>> {
        Err("SevenTrustHttpService: create_dynamic_qrcode not implemented".into())
    }
}
