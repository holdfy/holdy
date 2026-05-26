//! ImporterService: fetches the URL and runs extractors in cascade.
//! Redis cache (TTL 300 s) evita re-scraping da mesma URL dentro de 5 minutos.

use redis::{aio::ConnectionManager, AsyncCommands};
use reqwest::Client;
use url::Url;

use crate::error::ImporterError;
use crate::extractors::{
    Extractor, JsonLdExtractor, LlmExtractor, MercadoLivreExtractor, OpenGraphExtractor,
};
use crate::image_store::MinioImageStore;
use crate::types::ProductDraft;

const CACHE_TTL_SECS: u64 = 300;

/// Orchestrates the extractor pipeline with optional Redis cache.
pub struct ImporterService {
    client: Client,
    extractors: Vec<Box<dyn Extractor>>,
    image_store: Option<MinioImageStore>,
    redis: Option<ConnectionManager>,
}

impl ImporterService {
    pub fn new() -> Self {
        Self::build(None)
    }

    /// Async constructor: tries to connect to Redis from `REDIS_URL` env var.
    /// Falls back gracefully to no-cache if Redis is unavailable.
    pub async fn new_with_redis() -> Self {
        let redis = match std::env::var("REDIS_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
        {
            Some(url) => match redis::Client::open(url.trim().to_string()) {
                Ok(client) => match ConnectionManager::new(client).await {
                    Ok(mgr) => {
                        tracing::info!("importer: Redis cache ativado (TTL {}s)", CACHE_TTL_SECS);
                        Some(mgr)
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "importer: Redis indisponível, cache desativado");
                        None
                    }
                },
                Err(e) => {
                    tracing::warn!(error = %e, "importer: REDIS_URL inválido, cache desativado");
                    None
                }
            },
            None => {
                tracing::debug!("importer: REDIS_URL não configurado, cache desativado");
                None
            }
        };
        Self::build(redis)
    }

    fn build(redis: Option<ConnectionManager>) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; HoldfyBot/1.0; +https://holdfy.com/bot)")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest client");

        let extractors: Vec<Box<dyn Extractor>> = vec![
            Box::new(JsonLdExtractor),
            Box::new(OpenGraphExtractor),
            Box::new(MercadoLivreExtractor::new(client.clone())),
            Box::new(LlmExtractor::new(client.clone())),
        ];

        let image_store = MinioImageStore::from_env();
        if image_store.is_some() {
            tracing::info!("importer: MinIO image store enabled");
        }

        Self { client, extractors, image_store, redis }
    }

    /// Import a product from `url`.
    ///
    /// Checks Redis cache first (key `importer:v1:<url>`). On miss, fetches + extracts,
    /// then stores in cache. MinIO re-hosting applies before caching.
    pub async fn import(&self, url: &str) -> Result<ProductDraft, ImporterError> {
        let parsed = Url::parse(url).map_err(|_| ImporterError::InvalidUrl(url.to_string()))?;
        let scheme = parsed.scheme();
        if scheme != "https" && scheme != "http" {
            return Err(ImporterError::InvalidUrl(format!(
                "scheme '{scheme}' não suportado"
            )));
        }

        let cache_key = format!("importer:v1:{}", url);

        if let Some(cached) = self.cache_get(&cache_key).await {
            tracing::info!(url, "importer: cache hit");
            return Ok(cached);
        }

        tracing::info!(url, "importer: fetching");

        let html = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ImporterError::FetchFailed(e.to_string()))?
            .text()
            .await
            .map_err(|e| ImporterError::FetchFailed(e.to_string()))?;

        for extractor in &self.extractors {
            match extractor.extract(url, &html).await {
                Ok(Some(mut draft)) => {
                    tracing::info!(extractor = extractor.name(), url, "importer: extracted");
                    draft.photos = self.rehost_photos(draft.photos).await;
                    self.cache_set(&cache_key, &draft).await;
                    return Ok(draft);
                }
                Ok(None) => continue,
                Err(e) => {
                    tracing::warn!(extractor = extractor.name(), error = %e, "importer: extractor failed, trying next");
                }
            }
        }

        Err(ImporterError::NoDataExtracted)
    }

    async fn cache_get(&self, key: &str) -> Option<ProductDraft> {
        let Some(mut conn) = self.redis.clone() else {
            return None;
        };
        let val: Option<String> = conn.get(key).await.ok()?;
        let json = val?;
        match serde_json::from_str::<ProductDraft>(&json) {
            Ok(d) => Some(d),
            Err(e) => {
                tracing::warn!(error = %e, key, "importer: cache deserialize falhou");
                None
            }
        }
    }

    async fn cache_set(&self, key: &str, draft: &ProductDraft) {
        let Some(mut conn) = self.redis.clone() else {
            return;
        };
        match serde_json::to_string(draft) {
            Ok(json) => {
                if let Err(e) = conn.set_ex::<_, _, ()>(key, json, CACHE_TTL_SECS).await {
                    tracing::warn!(error = %e, key, "importer: cache set falhou");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "importer: serialização para cache falhou");
            }
        }
    }

    /// Upload all external photo URLs to MinIO. Silently skips failures (keeps original URL).
    async fn rehost_photos(&self, photos: Vec<String>) -> Vec<String> {
        let Some(store) = &self.image_store else {
            return photos;
        };
        let mut result = Vec::with_capacity(photos.len());
        for url in photos {
            match store.upload_from_url(&url).await {
                Ok(minio_url) => result.push(minio_url),
                Err(e) => {
                    tracing::warn!(url, error = %e, "importer: photo rehost failed, keeping original");
                    result.push(url);
                }
            }
        }
        result
    }
}

impl Default for ImporterService {
    fn default() -> Self {
        Self::new()
    }
}
