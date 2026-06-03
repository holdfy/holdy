//! Extrator dedicado para Facebook Marketplace e posts de produto.
//!
//! Estratégia em cascata:
//!   1. Scraper Service (Playwright headless) — preço, imagens, local, condição, vendedor
//!   2. HTTP mobile UA + og: tags + parse de description Marketplace
//!
//! og:description no Marketplace tem formato: "R$ X · Cidade, UF · Condição"

use async_trait::async_trait;
use reqwest::Client;

use crate::error::ImporterError;
use crate::types::{ProductDraft, SourcePlatform};

use super::Extractor;

const MOBILE_UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) \
    AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";

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
    video_url: Option<String>,
    seller_name: Option<String>,
    location: Option<String>,
    condition: Option<String>,
    product_id: Option<String>,
    canonical_url: Option<String>,
}

pub struct FacebookExtractor {
    client: Client,
}

impl FacebookExtractor {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn is_facebook_url(url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.contains("facebook.com") || lower.contains("fb.com")
    }

    async fn try_scraper_service(&self, url: &str) -> Option<ProductDraft> {
        let scraper_url = std::env::var("SCRAPER_URL").ok().filter(|s| !s.trim().is_empty())?;
        let api_key = std::env::var("SCRAPER_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var("APICASH_API_KEY").ok().filter(|s| !s.is_empty()));

        let endpoint = format!("{}/scrape", scraper_url.trim_end_matches('/'));

        let mut req = self
            .client
            .post(&endpoint)
            .json(&serde_json::json!({ "url": url }))
            .timeout(std::time::Duration::from_secs(45));

        if let Some(key) = api_key {
            req = req.header("x-api-key", key);
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!(error = %e, "facebook: scraper-service indisponível, usando fallback");
                return None;
            }
        };

        let scraped: ScraperResponse = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "facebook: scraper-service resposta inválida");
                return None;
            }
        };

        if !scraped.ok {
            tracing::warn!("facebook: scraper-service retornou ok=false");
            return None;
        }

        let data = scraped.data?;
        let title = data.title.filter(|s| !s.is_empty())?;

        let price_decimal = data.price.as_deref().and_then(normalize_price);

        let source_url = data.canonical_url.unwrap_or_else(|| url.to_string());

        tracing::info!(
            title = %title,
            images = data.images.len(),
            has_price = price_decimal.is_some(),
            "facebook: scraper-service extração completa"
        );

        Some(ProductDraft {
            title,
            description: data.description.filter(|s| !s.is_empty()),
            price_suggested: price_decimal,
            photos: data.images,
            video_url: data.video_url,
            source_url,
            source_platform: SourcePlatform::Facebook,
            extractor_used: "facebook_playwright".to_string(),
            guarantee: None,
            condition: data.condition,
            location: data.location,
            seller_name: data.seller_name,
            seller_rating: None,
            raw_attributes: serde_json::json!({
                "product_id": data.product_id,
                "platform": "facebook",
                "scraper": "playwright",
            }),
        })
    }

    async fn fetch_html_mobile(&self, url: &str) -> Option<String> {
        self.client
            .get(url)
            .header("User-Agent", MOBILE_UA)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "pt-BR,pt;q=0.9,en;q=0.8")
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()
    }

    fn parse_og_with_marketplace(url: &str, html: &str) -> Option<ProductDraft> {
        use scraper::{Html, Selector};

        let doc = Html::parse_document(html);
        let sel = Selector::parse("meta")
            .map_err(|e| tracing::warn!(error = %e, "facebook: seletor meta falhou"))
            .ok()?;

        let mut title = None::<String>;
        let mut description = None::<String>;
        let mut image = None::<String>;
        let mut video = None::<String>;

        for el in doc.select(&sel) {
            let prop = el
                .value()
                .attr("property")
                .or_else(|| el.value().attr("name"))
                .unwrap_or("");
            let content = el.value().attr("content").unwrap_or("").trim();
            match prop {
                "og:title" if title.is_none() && !content.is_empty() => title = Some(content.to_string()),
                "og:description" if description.is_none() && !content.is_empty() => description = Some(content.to_string()),
                "og:image" if image.is_none() && !content.is_empty() => image = Some(content.to_string()),
                "og:video" | "og:video:url" | "og:video:secure_url" if video.is_none() && !content.is_empty() => {
                    video = Some(content.to_string());
                }
                _ => {}
            }
        }

        let title = title?;

        let (price_decimal, location, condition) = description
            .as_deref()
            .map(parse_marketplace_description)
            .unwrap_or((None, None, None));

        let photos: Vec<String> = image.into_iter().collect();

        Some(ProductDraft {
            title,
            description: description.filter(|s| !s.is_empty()),
            price_suggested: price_decimal,
            photos,
            video_url: video,
            source_url: url.to_string(),
            source_platform: SourcePlatform::Facebook,
            extractor_used: "facebook_og".to_string(),
            guarantee: None,
            condition,
            location,
            seller_name: None,
            seller_rating: None,
            raw_attributes: serde_json::json!({"platform": "facebook", "fallback": "og_mobile"}),
        })
    }
}

/// Normaliza preço: "R$ 1.500,00" → Decimal
fn normalize_price(raw: &str) -> Option<rust_decimal::Decimal> {
    let digits: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
        .collect();
    let normalized = digits.replace(',', ".");
    let normalized = if normalized.matches('.').count() > 1 {
        let last = normalized.rfind('.')?;
        format!("{}.{}", normalized[..last].replace('.', ""), &normalized[last + 1..])
    } else {
        normalized
    };
    normalized.parse().ok()
}

/// Extrai (preço, local, condição) de "R$ 150 · São Paulo, SP · Usado"
fn parse_marketplace_description(
    desc: &str,
) -> (
    Option<rust_decimal::Decimal>,
    Option<String>,
    Option<String>,
) {
    let mut price = None;
    let mut location = None;
    let mut condition = None;

    for part in desc.split('·') {
        let part = part.trim();
        let lower = part.to_lowercase();

        if lower.starts_with("r$") || part.starts_with("R$") {
            price = normalize_price(part);
        } else if lower.contains("novo") {
            condition = Some("new".to_string());
        } else if lower.contains("recondicionado") {
            condition = Some("refurbished".to_string());
        } else if lower.contains("usado") {
            condition = Some("used".to_string());
        } else if part.contains(',') && part.split(',').count() == 2 {
            location = Some(part.to_string());
        }
    }

    (price, location, condition)
}

#[async_trait]
impl Extractor for FacebookExtractor {
    fn name(&self) -> &'static str {
        "facebook"
    }

    async fn extract(&self, url: &str, _html: &str) -> Result<Option<ProductDraft>, ImporterError> {
        if !Self::is_facebook_url(url) {
            return Ok(None);
        }

        // Estratégia 1: Playwright via scraper-service (dados completos)
        if let Some(draft) = self.try_scraper_service(url).await {
            return Ok(Some(draft));
        }

        // Estratégia 2: HTTP mobile UA + og: tags + parse Marketplace description
        tracing::debug!(url, "facebook: usando fallback og mobile");
        if let Some(html) = self.fetch_html_mobile(url).await {
            if let Some(draft) = Self::parse_og_with_marketplace(url, &html) {
                tracing::info!(
                    title = %draft.title,
                    has_price = draft.price_suggested.is_some(),
                    location = ?draft.location,
                    condition = ?draft.condition,
                    "facebook: og fallback extraction"
                );
                return Ok(Some(draft));
            }
        }

        Ok(None)
    }
}
