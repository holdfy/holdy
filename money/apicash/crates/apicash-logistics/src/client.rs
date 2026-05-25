//! HTTP client for the Melhor Envio API.

use reqwest::{Client, RequestBuilder};
use serde_json::Value;

use crate::error::LogisticsError;

const PRODUCTION_URL: &str = "https://melhorenvio.com.br/api/v2";
const SANDBOX_URL: &str = "https://sandbox.melhorenvio.com.br/api/v2";

pub struct MelhorEnvioClient {
    client: Client,
    base_url: &'static str,
    token: String,
}

impl Clone for MelhorEnvioClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            base_url: self.base_url,
            token: self.token.clone(),
        }
    }
}

impl MelhorEnvioClient {
    pub fn from_env() -> Result<Self, LogisticsError> {
        let token = std::env::var("MELHOR_ENVIO_TOKEN")
            .map_err(|_| LogisticsError::MissingToken)?;
        let sandbox = std::env::var("MELHOR_ENVIO_SANDBOX")
            .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
            .unwrap_or(true); // default: sandbox in dev

        let base_url = if sandbox { SANDBOX_URL } else { PRODUCTION_URL };

        Ok(Self {
            client: Client::new(),
            base_url,
            token,
        })
    }

    pub fn new(token: String, sandbox: bool) -> Self {
        Self {
            client: Client::new(),
            base_url: if sandbox { SANDBOX_URL } else { PRODUCTION_URL },
            token,
        }
    }

    fn auth(&self, rb: RequestBuilder) -> RequestBuilder {
        rb.bearer_auth(&self.token)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("User-Agent", "HoldfyApp/1.0 (saczuck.pos.ia@gmail.com)")
    }

    pub async fn post_json(&self, path: &str, body: &Value) -> Result<Value, LogisticsError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .auth(self.client.post(&url))
            .json(body)
            .send()
            .await
            .map_err(|e| LogisticsError::RequestFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(LogisticsError::ApiError(format!("HTTP {status}: {text}")));
        }

        resp.json::<Value>()
            .await
            .map_err(|e| LogisticsError::RequestFailed(e.to_string()))
    }

    pub async fn get_json(&self, path: &str) -> Result<Value, LogisticsError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .auth(self.client.get(&url))
            .send()
            .await
            .map_err(|e| LogisticsError::RequestFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(LogisticsError::ApiError(format!("HTTP {status}: {text}")));
        }

        resp.json::<Value>()
            .await
            .map_err(|e| LogisticsError::RequestFailed(e.to_string()))
    }
}
