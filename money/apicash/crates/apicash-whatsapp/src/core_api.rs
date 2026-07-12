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
    #[error("{0}")]
    Other(String),
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

/// Item de `GET /orders?role=buyer|seller` — campos exibidos no histórico WhatsApp.
#[derive(Debug, Deserialize)]
pub struct OrderListItem {
    pub id: Uuid,
    pub amount: String,
    pub status: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tracking_code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OrdersListResponse {
    orders: Vec<OrderListItem>,
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

    /// Lista pedidos do usuário. `role` = `"buyer"` ou `"seller"`. Retorna até 10 itens recentes.
    pub async fn list_orders(&self, role: &str, bearer: &str) -> Result<Vec<OrderListItem>, CoreApiError> {
        let resp: OrdersListResponse = self
            .get_json(&format!("/orders?role={role}"), Some(bearer))
            .await?;
        Ok(resp.orders)
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
        buyer_name: Option<&str>,
        bearer: Option<&str>,
    ) -> Result<CreateOrderResponse, CoreApiError> {
        let body = serde_json::json!({
            "buyer_id": buyer_id,
            "seller_id": seller_id,
            "amount": amount,
            "cpf": cpf,
            "social_links": social_links,
            "description": description,
            "buyer_name": buyer_name,
            "platform": "whatsapp",
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

    /// Off-ramp: envia BRLx ao vendedor via PIX após release do escrow.
    /// Requer que o pedido esteja `Completed` (custody liberada).
    pub async fn off_ramp_order(
        &self,
        order_id: Uuid,
        destination_pix_key: &str,
    ) -> Result<serde_json::Value, CoreApiError> {
        let body = serde_json::json!({ "destination_pix_key": destination_pix_key });
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        let url = format!("/orders/{order_id}/off-ramp");
        tracing::info!(%order_id, "core_api: POST /orders/:id/off-ramp");
        self.post_json_with_api_key(&url, &body, None, k).await
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

    /// Importa URL de anúncio via rota interna (service API key). Salva no banco e retorna dados + listing_id.
    pub async fn import_listing(&self, url: &str, user_id: Uuid) -> Result<ImportListingResponse, CoreApiError> {
        let body = serde_json::json!({ "url": url, "user_id": user_id });
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        tracing::info!(url_len = url.len(), %user_id, "core_api: POST /internal/listings/import");
        self.post_json_with_api_key("/internal/listings/import", &body, None, k).await
    }

    /// Enfileira importação assíncrona (Pulsar/NATS). Retorna `job_id` ou erro se fila não configurada.
    pub async fn import_listing_async(&self, url: &str, user_id: Uuid) -> Result<Uuid, CoreApiError> {
        let body = serde_json::json!({ "url": url, "user_id": user_id });
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        #[derive(serde::Deserialize)]
        struct AsyncResp { job_id: Uuid }
        let r: AsyncResp = self.post_json_with_api_key("/internal/listings/import/async", &body, None, k).await?;
        Ok(r.job_id)
    }

    /// Consulta status de um job. Quando `status == "done"` inclui dados do listing.
    pub async fn get_import_job(&self, job_id: Uuid) -> Result<ImportJobStatus, CoreApiError> {
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        let path = format!("/internal/listings/jobs/{job_id}");
        let url = format!("{}{}", self.base, path);
        let mut req = self.client.get(&url).headers(self.headers(None));
        if let Some(key) = k { req = req.header("x-api-key", key); }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(CoreApiError::Api { status: status.as_u16(), body: text });
        }
        Ok(serde_json::from_str(&text)?)
    }

    /// Polling assíncrono com timeout. Retorna dados do listing quando pronto, ou faz fallback para sync.
    /// Intervalo: 2s; timeout: 20s. Se a fila não estiver configurada, usa sync diretamente.
    pub async fn import_listing_with_queue_fallback(
        &self,
        url: &str,
        user_id: Uuid,
    ) -> Result<ImportListingResponse, CoreApiError> {
        match self.import_listing_async(url, user_id).await {
            Ok(job_id) => {
                // Polling com timeout de 20s
                for _ in 0..10 {
                    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                    match self.get_import_job(job_id).await {
                        Ok(job) if job.status == "done" => {
                            if let Some(listing) = job.listing {
                                return Ok(listing);
                            }
                            // Job done mas sem listing — fallback sync
                            break;
                        }
                        Ok(job) if job.status == "error" => {
                            return Err(CoreApiError::Other(
                                job.error_msg.unwrap_or_else(|| "import falhou na fila".into()),
                            ));
                        }
                        Ok(_) => continue,
                        Err(e) => {
                            tracing::warn!(error = %e, "poll import job failed — tentando sync");
                            break;
                        }
                    }
                }
                // Timeout ou job sem dados — fallback sync
                tracing::info!("import_listing: timeout no polling, usando sync");
                self.import_listing(url, user_id).await
            }
            Err(_) => {
                // Fila não configurada — usa sync diretamente
                self.import_listing(url, user_id).await
            }
        }
    }

    /// Retorna o dispute_id para um pedido (None se não houver disputa aberta).
    pub async fn get_dispute_for_order(&self, order_id: Uuid) -> Result<Option<Uuid>, CoreApiError> {
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        let url = format!("/orders/{order_id}/dispute");
        let mut req = self.client.get(format!("{}{}", self.base, url))
            .headers(self.headers(None));
        if let Some(key) = k { req = req.header("x-api-key", key); }
        let resp = req.send().await.map_err(CoreApiError::from)?;
        if resp.status().as_u16() == 404 { return Ok(None); }
        if !resp.status().is_success() {
            return Err(CoreApiError::Other(format!("dispute HTTP {}", resp.status())));
        }
        let val: serde_json::Value = resp.json().await.map_err(CoreApiError::from)?;
        let id = val["dispute_id"].as_str()
            .and_then(|s| Uuid::parse_str(s).ok());
        Ok(id)
    }

    /// Registra evidência de mídia já enviada ao MinIO (URL + SHA-256 pre-computados).
    pub async fn add_dispute_evidence_media(
        &self,
        order_id: Uuid,
        minio_url: &str,
        sha256: &str,
        kind: &str,
    ) -> Result<serde_json::Value, CoreApiError> {
        let body = serde_json::json!({
            "kind": kind,
            "content": minio_url,    // URL pública MinIO
            "sha256_override": sha256,
        });
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        self.post_json_with_api_key(&format!("/orders/{order_id}/dispute/evidence"), &body, None, k).await
    }

    /// Abre disputa para um pedido.
    pub async fn open_dispute(&self, order_id: Uuid, reason: &str) -> Result<serde_json::Value, CoreApiError> {
        let body = serde_json::json!({ "reason": reason });
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        self.post_json_with_api_key(&format!("/orders/{order_id}/dispute"), &body, None, k).await
    }

    /// Adiciona evidência textual (rastreio, mensagem) a uma disputa.
    pub async fn add_dispute_evidence_text(
        &self,
        order_id: Uuid,
        _uploaded_by: Uuid,
        content: &str,
    ) -> Result<serde_json::Value, CoreApiError> {
        let body = serde_json::json!({ "kind": "message", "content": content });
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        self.post_json_with_api_key(&format!("/orders/{order_id}/dispute/evidence"), &body, None, k).await
    }

    /// Aciona análise de evidências pela IA (fire-and-forget).
    pub async fn trigger_dispute_analysis(&self, order_id: Uuid) -> Result<serde_json::Value, CoreApiError> {
        let body = serde_json::json!({});
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        self.post_json_with_api_key(&format!("/orders/{order_id}/dispute/analyze"), &body, None, k).await
    }

    /// Consulta reputação de um usuário (`GET /reputation/{user_id}`). Soft failure — retorna None se falhar.
    pub async fn get_reputation(&self, user_id: Uuid, bearer: &str) -> Option<ReputationResponse> {
        self.get_json(&format!("/reputation/{user_id}"), Some(bearer))
            .await
            .ok()
    }

    /// Vincula um listing a um pedido via rota interna (fire-and-forget aceitável).
    pub async fn link_listing_to_order(&self, listing_id: Uuid, order_id: Uuid) -> Result<(), CoreApiError> {
        let body = serde_json::json!({ "order_id": order_id });
        let k = std::env::var("APICASH_API_KEY").ok();
        let k = k.as_deref().filter(|s| !s.is_empty());
        let url = format!("/internal/listings/{listing_id}/order");
        let mut req = self.client.patch(format!("{}{}", self.base, url))
            .headers(self.headers(None))
            .json(&body);
        if let Some(key) = k {
            req = req.header("x-api-key", key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            tracing::warn!(listing_id = %listing_id, order_id = %order_id, status = %resp.status(), "link_listing_to_order: non-2xx");
        }
        Ok(())
    }
}

/// Status de um job de importação assíncrona (`GET /v1/listings/jobs/{id}`).
#[derive(Debug, Deserialize)]
pub struct ImportJobStatus {
    pub job_id: String,
    pub status: String,
    pub error_msg: Option<String>,
    pub listing: Option<ImportListingResponse>,
}

/// Resposta de `GET /reputation/{user_id}`.
#[derive(Debug, Deserialize, Clone)]
pub struct ReputationResponse {
    pub score: u32,
    pub seal: Option<ReputationSeal>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReputationSeal {
    pub name: String,
    pub label: String,
}

impl ReputationSeal {
    /// Emoji + label para exibição no WhatsApp.
    pub fn whatsapp_display(&self) -> String {
        let emoji = match self.name.as_str() {
            "premium" => "⭐",
            "authenticated" => "✅",
            _ => "🔵",
        };
        format!("{emoji} *{}*", self.label)
    }
}

/// Resposta de `POST /v1/listings/import`.
#[derive(Debug, Deserialize)]
pub struct ImportListingResponse {
    pub listing_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub price_suggested: Option<String>,
    #[serde(default)]
    pub photos: Vec<String>,
    pub source_url: String,
    pub source_platform: String,
    pub seller_name: Option<String>,
    pub video_url: Option<String>,
}
