// Wraps a GatewayHttpService and caches get_token_out via Redis TokenCache
use std::sync::Arc;

use async_trait::async_trait;

use crate::core::gateways::interfaces::sulcred::SulcredSendPixKeyRequest;
use crate::core::redis::TokenCache;
use crate::core::rabbitmq::GatewayFailureConfig;

use super::traits::{
    AuthOutResponse, BalanceResponse, CreateDynamicQrcodeRequest, CreateDynamicQrcodeResponse,
    GatewayHttpService, SendPixKeyResponse,
};

/// Gateway name for cache key (e.g. "blindpay", "sulcred", "seventrust").
pub fn gateway_name_blindpay() -> &'static str { "blindpay" }
pub fn gateway_name_sulcred() -> &'static str { "sulcred" }
pub fn gateway_name_seventrust() -> &'static str { "seventrust" }
pub fn gateway_name_next() -> &'static str { "next" }

/// Wraps a GatewayHttpService and caches OAuth token in Redis.
pub struct GatewayWithTokenCache {
    inner: Arc<dyn GatewayHttpService>,
    token_cache: Arc<TokenCache>,
    gateway_name: String,
}

impl GatewayWithTokenCache {
    pub fn new(inner: Arc<dyn GatewayHttpService>, token_cache: Arc<TokenCache>, gateway_name: String) -> Self {
        Self { inner, token_cache, gateway_name }
    }

    /// Returns cached token or fetches and caches.
    pub async fn get_token_out_cached(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<AuthOutResponse, Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(tok) = self.token_cache.get_gateway_token(&self.gateway_name, client_id).await {
            if !tok.is_empty() {
                return Ok(AuthOutResponse {
                    access_token: tok,
                    expires_in: 3600,
                    token_type: Some("Bearer".to_string()),
                });
            }
        }
        let resp = self.inner.get_token_out(client_id, client_secret).await?;
        let ttl = std::time::Duration::from_secs((resp.expires_in as u64).saturating_sub(60));
        let _ = self
            .token_cache
            .set_gateway_token(&self.gateway_name, client_id, &resp.access_token, Some(ttl))
            .await;
        Ok(resp)
    }
}

#[async_trait]
impl GatewayHttpService for GatewayWithTokenCache {
    async fn get_token_out(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<AuthOutResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.get_token_out_cached(client_id, client_secret).await
    }

    async fn send_pix_key(
        &self,
        token: &str,
        request: &SulcredSendPixKeyRequest,
        failure_config: Option<&GatewayFailureConfig>,
    ) -> Result<SendPixKeyResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.send_pix_key(token, request, failure_config).await
    }

    async fn get_balance(&self, token: &str) -> Result<BalanceResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.get_balance(token).await
    }

    async fn create_dynamic_qrcode(
        &self,
        token: &str,
        request: &CreateDynamicQrcodeRequest,
    ) -> Result<CreateDynamicQrcodeResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.create_dynamic_qrcode(token, request).await
    }
}
