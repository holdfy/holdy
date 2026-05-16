//! Cliente HTTP para a API principal (`apicash-core`).

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone)]
pub struct CoreApiClient {
    base: String,
    client: reqwest::Client,
}

#[derive(Debug, Error)]
pub enum CoreApiError {
    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API {status}: {body}")]
    Api { status: u16, body: String },
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// Corpo JSON de `POST /orders` (espelha a API `apicash-core`).
#[derive(Debug, Deserialize)]
pub struct CreateOrderResponse {
    pub id: Uuid,
    pub status: String,
    pub fiat_rail: String,
    #[serde(default)]
    pub anchor_tx_hash: Option<String>,
    #[serde(default)]
    pub gateway_in_tx_id: Option<String>,
    #[serde(default)]
    pub funding_reference: Option<String>,
    #[serde(default)]
    pub pix_br_code: Option<String>,
    #[serde(default)]
    pub funding_instruction: Option<String>,
}

/// Corpo JSON de `POST /payments/pix`.
#[derive(Debug, Deserialize)]
pub struct PixPaymentResponse {
    pub stellar_tx_hash: String,
    pub status: String,
    pub pix_br_code: String,
}

/// Resposta mínima de `GET /orders/{id}` (usada para validação de segurança antes do release).
#[derive(Debug, Deserialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub status: String,
}

/// Resposta mínima de score (`POST /risk/score` / `/internal/risk/score`).
#[derive(Debug, Deserialize)]
pub struct RiskScoreResponse {
    pub score: u32,
    pub decision: serde_json::Value,
    pub risk_level: serde_json::Value,
}

impl CoreApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base: base_url.into().trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self, bearer: Option<&str>) -> HeaderMap {
        let mut h = HeaderMap::new();
        if let Some(t) = bearer {
            if let Ok(v) = HeaderValue::from_str(&format!("Bearer {t}")) {
                h.insert(AUTHORIZATION, v);
            }
        }
        h
    }

    fn headers_with_api_key(&self, bearer: Option<&str>, api_key: Option<&str>) -> HeaderMap {
        let mut h = self.headers(bearer);
        if let Some(k) = api_key {
            if let Ok(v) = HeaderValue::from_str(k) {
                h.insert("x-api-key", v);
            }
        }
        h
    }

    async fn post_json<T: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &T,
        bearer: Option<&str>,
    ) -> Result<R, CoreApiError> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .client
            .post(&url)
            .headers(self.headers(bearer))
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(CoreApiError::Api {
                status: status.as_u16(),
                body: text,
            });
        }
        Ok(serde_json::from_str(&text)?)
    }

    async fn post_json_with_api_key<T: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &T,
        bearer: Option<&str>,
        api_key: Option<&str>,
    ) -> Result<R, CoreApiError> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .client
            .post(&url)
            .headers(self.headers_with_api_key(bearer, api_key))
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(CoreApiError::Api {
                status: status.as_u16(),
                body: text,
            });
        }
        Ok(serde_json::from_str(&text)?)
    }

    async fn get_json<R: DeserializeOwned>(
        &self,
        path: &str,
        bearer: Option<&str>,
    ) -> Result<R, CoreApiError> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers(bearer))
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(CoreApiError::Api {
                status: status.as_u16(),
                body: text,
            });
        }
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn get_order(
        &self,
        order_id: Uuid,
        bearer: Option<&str>,
    ) -> Result<OrderResponse, CoreApiError> {
        self.get_json(&format!("/orders/{order_id}"), bearer).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_order(
        &self,
        buyer_id: Uuid,
        seller_id: Uuid,
        amount: &str,
        cpf: &str,
        social_links: &[String],
        description: Option<&str>,
        bearer: Option<&str>,
    ) -> Result<CreateOrderResponse, CoreApiError> {
        let body = serde_json::json!({
            "buyer_id": buyer_id,
            "seller_id": seller_id,
            "amount": amount,
            "cpf": cpf,
            "social_links": social_links,
            "description": description,
        });
        tracing::info!(%buyer_id, %seller_id, %amount, "core_api: POST /orders (pedido protegido + antifraude)");
        self.post_json("/orders", &body, bearer).await
    }

    pub async fn calculate_risk_score(
        &self,
        user_id: Uuid,
        cpf: &str,
        social_links: &[String],
        bearer: Option<&str>,
    ) -> Result<RiskScoreResponse, CoreApiError> {
        let body = serde_json::json!({
            "user_id": user_id,
            "cpf": cpf,
            "social_links": social_links,
        });

        // Security decision:
        // - Prefer the user JWT-bound route (`POST /risk/score`) when we have a valid bearer token.
        // - Fallback to the internal service route (`POST /internal/risk/score`) guarded by
        //   `APICASH_API_KEY` (X-API-Key) if the public route returns 401 and a key is configured.
        tracing::info!(%user_id, "core_api: POST /risk/score (antifraude pré-cálculo)");
        match self.post_json("/risk/score", &body, bearer).await {
            Ok(v) => Ok(v),
            Err(CoreApiError::Api { status: 401, .. }) => {
                let k = std::env::var("APICASH_API_KEY").ok();
                let k = k.as_deref().filter(|s| !s.is_empty());
                if k.is_none() {
                    return Err(CoreApiError::Api {
                        status: 401,
                        body:
                            "unauthorized and APICASH_API_KEY not configured for internal fallback"
                                .into(),
                    });
                }
                tracing::warn!(%user_id, "core_api: /risk/score unauthorized; trying /internal/risk/score with service key");
                self.post_json_with_api_key("/internal/risk/score", &body, None, k)
                    .await
            }
            Err(e) => Err(e),
        }
    }

    /// Antifraude só com chave de serviço (fluxo vendedor-inicia no WhatsApp).
    pub async fn calculate_risk_score_internal_only(
        &self,
        user_id: Uuid,
        cpf: &str,
        social_links: &[String],
    ) -> Result<serde_json::Value, CoreApiError> {
        let body = serde_json::json!({
            "user_id": user_id,
            "cpf": cpf,
            "social_links": social_links,
        });
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        let Some(key) = k else {
            return Err(CoreApiError::Api {
                status: 500,
                body: "APICASH_API_KEY required for internal risk score (seller-initiated flow)"
                    .into(),
            });
        };
        tracing::info!(%user_id, "core_api: POST /internal/risk/score (service)");
        self.post_json_with_api_key("/internal/risk/score", &body, None, Some(key))
            .await
    }

    pub async fn create_pix(
        &self,
        user_id: Uuid,
        amount: &str,
        cpf: &str,
        social_links: &[String],
        bearer: Option<&str>,
    ) -> Result<PixPaymentResponse, CoreApiError> {
        let body = serde_json::json!({
            "user_id": user_id,
            "amount": amount,
            "cpf": cpf,
            "social_links": social_links,
        });
        tracing::info!(%user_id, %amount, "core_api: POST /payments/pix (instrução PIX via anchor)");
        self.post_json("/payments/pix", &body, bearer).await
    }

    pub async fn release_custody(
        &self,
        order_id: Uuid,
        released_by: Uuid,
        idempotency_key: &str,
        bearer: Option<&str>,
    ) -> Result<serde_json::Value, CoreApiError> {
        let body = serde_json::json!({
            "order_id": order_id,
            "released_by": released_by,
            "idempotency_key": idempotency_key,
        });
        tracing::info!(%order_id, %released_by, "core_api: POST /custody/release");
        self.post_json("/custody/release", &body, bearer).await
    }

    pub async fn settle_order_internal(
        &self,
        order_id: Uuid,
        bearer: Option<&str>,
    ) -> Result<serde_json::Value, CoreApiError> {
        let body = serde_json::json!({ "order_id": order_id });
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        self.post_json_with_api_key("/internal/orders/settle", &body, bearer, k)
            .await
    }
}
