//! Extrator dedicado para OLX Brasil.
//!
//! OLX bloqueia scraping HTTP simples via challenge Cloudflare (confirmado: mesmo
//! com User-Agent de browser real, a resposta é sempre 403 "Just a moment...").
//! Não existe fallback sem JavaScript — a única via é o scraper-service
//! (Playwright + stealth), que passa pelo challenge e lê o JSON-LD `schema.org/Product`
//! já renderizado na página (título, descrição, preço, imagens).

use async_trait::async_trait;
use reqwest::Client;

use crate::error::ImporterError;
use crate::types::{ProductDraft, SourcePlatform};

use super::Extractor;

/// Resultado do scraper-service (Playwright).
#[derive(serde::Deserialize, Debug)]
struct ScraperResponse {
    ok: bool,
    data: Option<ScraperData>,
}

#[derive(serde::Deserialize, Debug)]
struct ScraperData {
    title: Option<String>,
    description: Option<String>,
    price: Option<String>,
    #[serde(default)]
    images: Vec<String>,
    seller_name: Option<String>,
    location: Option<String>,
    product_id: Option<String>,
}

pub struct OlxExtractor {
    client: Client,
}

impl OlxExtractor {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn is_olx_url(url: &str) -> bool {
        url.to_lowercase().contains("olx.com")
    }

    /// Tenta usar o scraper-service (Playwright) para obter dados completos.
    /// Retorna `None` se o serviço não estiver configurado ou falhar — não há
    /// fallback HTTP puro possível (ver módulo doc).
    async fn try_scraper_service(&self, url: &str) -> Option<ProductDraft> {
        let scraper_url = std::env::var("SCRAPER_URL").ok().filter(|s| !s.trim().is_empty())?;
        let api_key = std::env::var("SCRAPER_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var("APICASH_API_KEY").ok().filter(|s| !s.is_empty()));

        let endpoint = format!("{}/scrape", scraper_url.trim_end_matches('/'));
        let body = serde_json::json!({ "url": url });

        let mut req = self
            .client
            .post(&endpoint)
            .json(&body)
            .timeout(std::time::Duration::from_secs(75)); // Playwright + challenge Cloudflare + retentativa de boot do Chromium pode demorar

        if let Some(key) = api_key {
            req = req.header("x-api-key", key);
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!(error = %e, "olx: scraper-service indisponível");
                return None;
            }
        };

        let scraped: ScraperResponse = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "olx: scraper-service resposta inválida");
                return None;
            }
        };

        if !scraped.ok {
            tracing::warn!("olx: scraper-service retornou ok=false");
            return None;
        }

        let data = scraped.data?;
        let title = data.title.filter(|s| !s.is_empty())?;

        // Preço vem cru do JSON-LD (ex: "6500"), sem formatação de milhar.
        let price_decimal = data
            .price
            .as_deref()
            .and_then(|p| p.replace(',', ".").parse::<rust_decimal::Decimal>().ok());

        tracing::info!(
            title = %title,
            images = data.images.len(),
            has_price = price_decimal.is_some(),
            "olx: scraper-service extração completa"
        );

        Some(ProductDraft {
            title,
            description: data.description.filter(|s| !s.is_empty()),
            price_suggested: price_decimal,
            photos: data.images,
            video_url: None,
            source_url: url.to_string(),
            source_platform: SourcePlatform::Olx,
            extractor_used: "olx_playwright".to_string(),
            guarantee: None,
            condition: None,
            location: data.location,
            seller_name: data.seller_name,
            seller_rating: None,
            raw_attributes: serde_json::json!({
                "product_id": data.product_id,
                "platform": "olx",
                "scraper": "playwright",
            }),
        })
    }
}

#[async_trait]
impl Extractor for OlxExtractor {
    fn name(&self) -> &'static str {
        "olx"
    }

    async fn extract(&self, url: &str, _html: &str) -> Result<Option<ProductDraft>, ImporterError> {
        if !Self::is_olx_url(url) {
            return Ok(None);
        }

        Ok(self.try_scraper_service(url).await)
    }
}
