use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::gateways::interfaces::sulcred::SulcredSendPixPayment;
use crate::core::gateways::services::GatewayHttpService;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPixRequest {
    pub account: String,
    pub bank: String,
    pub document_number: String,
    pub amount: f64,
    pub branch: String,
    pub key: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_key: Option<String>,
    /// Required for async flow: authentication_id (user_id) to resolve account.
    #[serde(rename = "userId", default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPixResponse {
    pub status_code: i32,
    pub transaction_id: String,
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateQrCodeRequest {
    pub amount: f64,
    pub payer_name: String,
    pub payer_document: String,
    pub description: String,
    pub expiration_seconds: i32,
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pix_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateQrCodeResponse {
    pub status_code: i32,
    pub qr_code: String,
    pub tx_id: String,
    pub expires_at: String,
    pub transaction_id: String,
    pub gateway: String,
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// QR sintético quando não há PSP externo (stub / async dev). Não é BR Code EMV para cobrança real.
pub(crate) fn synthetic_pix_qrcode_response(
    req: GenerateQrCodeRequest,
    gateway_label: &str,
) -> Result<GenerateQrCodeResponse, Box<dyn std::error::Error + Send + Sync>> {
    if req.amount <= 0.0 {
        return Err(anyhow::anyhow!("amount must be positive").into());
    }
    let tx_id = format!(
        "TX{}",
        chrono::Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_else(|| chrono::Utc::now().timestamp_micros())
    );
    let expires_at =
        (chrono::Utc::now() + chrono::Duration::seconds(req.expiration_seconds as i64)).to_rfc3339();
    // Referência HoldFy (`order:uuid`) embutida para o banco notificar WhatsApp após pagamento.
    let ref_token = req.reference.trim().replace(':', "_");
    let qr_code = format!("GATEBOXRUST:QR|{ref_token}|{:.2}", req.amount);
    let mut data = std::collections::HashMap::new();
    data.insert("reference".to_string(), serde_json::Value::String(req.reference.clone()));
    data.insert(
        "amount".to_string(),
        serde_json::Value::Number(
            serde_json::Number::from_f64(req.amount).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
    );
    if let Some(pk) = req.pix_key.as_ref() {
        data.insert("pixKey".to_string(), serde_json::Value::String(pk.clone()));
    }
    Ok(GenerateQrCodeResponse {
        status_code: 200,
        qr_code,
        tx_id: tx_id.clone(),
        expires_at,
        transaction_id: tx_id,
        gateway: gateway_label.to_string(),
        data,
    })
}

#[async_trait]
pub trait PixPrincipalService: Send + Sync {
    async fn send_pix(&self, req: SendPixRequest) -> Result<SendPixResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn generate_qr_code(&self, req: GenerateQrCodeRequest) -> Result<GenerateQrCodeResponse, Box<dyn std::error::Error + Send + Sync>>;
}

/// Stub implementation for wiring (no real PIX).
pub struct PixPrincipalServiceStub;

#[async_trait]
impl PixPrincipalService for PixPrincipalServiceStub {
    async fn send_pix(&self, _req: SendPixRequest) -> Result<SendPixResponse, Box<dyn std::error::Error + Send + Sync>> {
        Ok(SendPixResponse {
            status_code: 200,
            transaction_id: String::new(),
            data: std::collections::HashMap::new(),
        })
    }
    async fn generate_qr_code(&self, req: GenerateQrCodeRequest) -> Result<GenerateQrCodeResponse, Box<dyn std::error::Error + Send + Sync>> {
        synthetic_pix_qrcode_response(req, "stub")
    }
}

/// Real implementation that delegates send_pix to a gateway (e.g. Sulcred).
pub struct PixPrincipalServiceImpl {
    gateway: Arc<dyn GatewayHttpService>,
    client_id: String,
    client_secret: String,
}

impl PixPrincipalServiceImpl {
    pub fn new(gateway: Arc<dyn GatewayHttpService>, client_id: String, client_secret: String) -> Self {
        PixPrincipalServiceImpl {
            gateway,
            client_id,
            client_secret,
        }
    }
}

#[async_trait]
impl PixPrincipalService for PixPrincipalServiceImpl {
    async fn send_pix(&self, req: SendPixRequest) -> Result<SendPixResponse, Box<dyn std::error::Error + Send + Sync>> {
        let token_resp = self
            .gateway
            .get_token_out(&self.client_id, &self.client_secret)
            .await?;
        let token = token_resp.access_token;
        let gw_request = crate::core::gateways::interfaces::sulcred::SulcredSendPixKeyRequest {
            expiration: 3600,
            pix_key: req.key.clone(),
            priority: None,
            creditor_document: Some(req.document_number.clone()),
            description: req.memo.clone().unwrap_or_default(),
            payment: SulcredSendPixPayment {
                currency: "BRL".to_string(),
                amount: req.amount,
            },
            internal_transaction_id: req.external_id.clone(),
        };
        let gw_resp = self.gateway.send_pix_key(&token, &gw_request, None).await?;
        let mut data = std::collections::HashMap::new();
        data.insert("endToEndId".to_string(), serde_json::Value::String(gw_resp.end_to_end_id.clone()));
        data.insert("status".to_string(), serde_json::Value::String(gw_resp.status.clone()));
        Ok(SendPixResponse {
            status_code: 200,
            transaction_id: gw_resp.end_to_end_id,
            data,
        })
    }

    async fn generate_qr_code(&self, req: GenerateQrCodeRequest) -> Result<GenerateQrCodeResponse, Box<dyn std::error::Error + Send + Sync>> {
        let token_resp = self
            .gateway
            .get_token_out(&self.client_id, &self.client_secret)
            .await?;
        let token = token_resp.access_token;
        let gw_req = crate::core::gateways::services::CreateDynamicQrcodeRequest {
            payer_name: req.payer_name,
            payer_document: req.payer_document,
            description: req.description,
            amount: req.amount,
            expiration_seconds: req.expiration_seconds,
            pix_key: req.pix_key.unwrap_or_default(),
            reference: Some(req.reference),
        };
        let gw_resp = self.gateway.create_dynamic_qrcode(&token, &gw_req).await?;
        let mut data = std::collections::HashMap::new();
        data.insert("txId".to_string(), serde_json::Value::String(gw_resp.txid.clone().unwrap_or_default()));
        data.insert("location".to_string(), serde_json::Value::String(gw_resp.location.clone().unwrap_or_default()));
        Ok(GenerateQrCodeResponse {
            status_code: 200,
            qr_code: gw_resp.pix_copia_e_cola,
            tx_id: gw_resp.txid.unwrap_or_default(),
            expires_at: String::new(),
            transaction_id: String::new(),
            gateway: "sulcred".to_string(),
            data,
        })
    }
}
