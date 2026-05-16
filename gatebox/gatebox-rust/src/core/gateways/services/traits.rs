// Trait for gateway HTTP services (Sulcred, SevenTrust, Next, etc.)
use async_trait::async_trait;

use crate::core::gateways::interfaces::sulcred::{
    SulcredSendPixKeyRequest, SulcredSendPixKeyResponse,
};
use crate::core::rabbitmq::GatewayFailureConfig;

/// OAuth token response (common shape for gateways).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthOutResponse {
    pub access_token: String,
    pub expires_in: i32,
    #[serde(rename = "token_type")]
    pub token_type: Option<String>,
}

/// Balance response (common shape).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BalanceResponse {
    pub balance: Option<f64>,
    pub available: Option<f64>,
}

/// Send PIX response - we use Sulcred shape as reference; others can map.
pub type SendPixKeyResponse = SulcredSendPixKeyResponse;

/// Dynamic QR Code request (PIX IN - create cob).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateDynamicQrcodeRequest {
    pub payer_name: String,
    pub payer_document: String,
    pub description: String,
    pub amount: f64,
    pub expiration_seconds: i32,
    pub pix_key: String,
    pub reference: Option<String>,
}

/// Dynamic QR Code response (pixCopiaECola = QR code string).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateDynamicQrcodeResponse {
    #[serde(rename = "pixCopiaECola")]
    pub pix_copia_e_cola: String,
    pub txid: Option<String>,
    pub location: Option<String>,
    pub status: Option<String>,
}

/// Gateway HTTP service: auth, send PIX, balance, create dynamic QR (PIX IN).
#[async_trait]
pub trait GatewayHttpService: Send + Sync {
    /// Get OAuth token for PIX OUT (or equivalent).
    async fn get_token_out(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<AuthOutResponse, Box<dyn std::error::Error + Send + Sync>>;

    /// Send PIX by key.
    async fn send_pix_key(
        &self,
        token: &str,
        request: &SulcredSendPixKeyRequest,
        failure_config: Option<&GatewayFailureConfig>,
    ) -> Result<SendPixKeyResponse, Box<dyn std::error::Error + Send + Sync>>;

    /// Get account balance.
    async fn get_balance(&self, token: &str) -> Result<BalanceResponse, Box<dyn std::error::Error + Send + Sync>>;

    /// Create dynamic PIX QR Code (PIX IN - cob).
    async fn create_dynamic_qrcode(
        &self,
        token: &str,
        request: &CreateDynamicQrcodeRequest,
    ) -> Result<CreateDynamicQrcodeResponse, Box<dyn std::error::Error + Send + Sync>> {
        let _ = (token, request);
        Err("create_dynamic_qrcode not implemented".into())
    }
}
