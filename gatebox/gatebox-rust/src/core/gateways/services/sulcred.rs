// Sulcred HTTP service (from app/modules/core/gateways/services/sulcred)
use async_trait::async_trait;
use reqwest::Client;

use crate::core::gateways::interfaces::sulcred::{
    SulcredCreateDynamicBrcodeRequest, SulcredCreateDynamicBrcodeResponse, SulcredDevedor,
    SulcredSendPixKeyRequest, SulcredSendPixKeyResponse, SulcredValor,
};
use crate::core::gateways::services::traits::{
    AuthOutResponse, BalanceResponse, CreateDynamicQrcodeRequest, CreateDynamicQrcodeResponse,
    GatewayHttpService, SendPixKeyResponse,
};
use crate::core::rabbitmq::GatewayFailureConfig;

fn base_url_out() -> String {
    std::env::var("SULCRED_OUT_URL").unwrap_or_else(|_| "https://api.sulcred.example".to_string())
}

/// Sulcred HTTP service for PIX OUT (token, send, balance).
pub struct SulcredHttpService {
    client: Client,
    base_url_out: String,
}

impl Default for SulcredHttpService {
    fn default() -> Self {
        Self::new(Client::new(), base_url_out())
    }
}

impl SulcredHttpService {
    pub fn new(client: Client, base_url_out: String) -> Self {
        SulcredHttpService { client, base_url_out }
    }

    pub fn with_base_url_out(mut self, url: String) -> Self {
        self.base_url_out = url;
        self
    }
}

#[async_trait]
impl GatewayHttpService for SulcredHttpService {
    async fn get_token_out(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<AuthOutResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v2/oauth/token", self.base_url_out.trim_end_matches('/'));
        let body = serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "grant_type": "client_credentials"
        });
        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("request: {}", e))?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {}: {}", status, text).into());
        }
        let auth: AuthOutResponse = res.json().await.map_err(|e| anyhow::anyhow!("json: {}", e))?;
        Ok(auth)
    }

    async fn send_pix_key(
        &self,
        token: &str,
        request: &SulcredSendPixKeyRequest,
        failure_config: Option<&GatewayFailureConfig>,
    ) -> Result<SendPixKeyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v2/pix/payments/dict", self.base_url_out.trim_end_matches('/'));
        let mut req = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(request);
        if let Some(cfg) = failure_config {
            if let Ok(json) = serde_json::to_string(cfg) {
                req = req.header("X-Gateway-Failure-Config", json);
            }
        }
        let res = req.send().await.map_err(|e| anyhow::anyhow!("request: {}", e))?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {}: {}", status, text).into());
        }
        let out: SulcredSendPixKeyResponse = res.json().await.map_err(|e| anyhow::anyhow!("json: {}", e))?;
        Ok(out)
    }

    async fn get_balance(&self, token: &str) -> Result<BalanceResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v2/accounts/balances/", self.base_url_out.trim_end_matches('/'));
        let res = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("request: {}", e))?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {}: {}", status, text).into());
        }
        let raw: serde_json::Value = res.json().await.map_err(|e| anyhow::anyhow!("json: {}", e))?;
        let balance = raw.get("balance").and_then(|v| v.as_f64());
        let available = raw.get("available").and_then(|v| v.as_f64()).or(balance);
        Ok(BalanceResponse {
            balance,
            available,
        })
    }

    async fn create_dynamic_qrcode(
        &self,
        token: &str,
        request: &CreateDynamicQrcodeRequest,
    ) -> Result<CreateDynamicQrcodeResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Sulcred PIX IN uses "cob" on base URL IN; we use base_url_out for simplicity if same host
        let url = format!("{}/cob", self.base_url_out.trim_end_matches('/'));
        let body = SulcredCreateDynamicBrcodeRequest {
            calendario: Some(crate::core::gateways::interfaces::sulcred::SulcredCalendario {
                expiracao: Some(request.expiration_seconds),
            }),
            devedor: SulcredDevedor {
                cpf: if request.payer_document.len() == 11 {
                    Some(request.payer_document.clone())
                } else {
                    None
                },
                cnpj: if request.payer_document.len() == 14 {
                    Some(request.payer_document.clone())
                } else {
                    None
                },
                nome: request.payer_name.clone(),
            },
            valor: SulcredValor {
                original: format!("{:.2}", request.amount),
                modalidade_alteracao: 0,
            },
            chave: request.pix_key.clone(),
            solicitacao_pagador: request.description.clone(),
        };
        let res = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("request: {}", e))?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("HTTP {}: {}", status, text).into());
        }
        let out: SulcredCreateDynamicBrcodeResponse = res.json().await.map_err(|e| anyhow::anyhow!("json: {}", e))?;
        Ok(CreateDynamicQrcodeResponse {
            pix_copia_e_cola: out.pix_copia_e_cola,
            txid: out.txid,
            location: None,
            status: out.status,
        })
    }
}
