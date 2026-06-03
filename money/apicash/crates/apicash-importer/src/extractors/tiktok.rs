//! Extrator dedicado para TikTok Shop (vt.tiktok.com / www.tiktok.com/view/product).
//!
//! Estratégia em cascata:
//!   1. Scraper Service (Playwright headless) — preço, 9+ imagens, descrição completa
//!   2. Redirect og_info fallback — título + 1 imagem (sem JS)
//!
//! Dados disponíveis via redirect (sem scraper):
//!   - title, image (1), seller_name (unique_id), product_id
//!
//! Dados disponíveis via Playwright:
//!   - title, description, price, images (todas), seller_name, seller_rating

use async_trait::async_trait;
use reqwest::Client;
use url::Url;

use crate::error::ImporterError;
use crate::types::{ProductDraft, SourcePlatform};

use super::Extractor;

const MOBILE_UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) \
    AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

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
    price_original: Option<String>,
    #[serde(default)]
    images: Vec<String>,
    video_url: Option<String>,
    seller_name: Option<String>,
    seller_rating: Option<String>,
    product_id: Option<String>,
}

pub struct TikTokExtractor {
    client: Client,
}

impl TikTokExtractor {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Tenta usar o scraper-service (Playwright) para obter dados completos.
    /// Retorna None se o serviço não estiver configurado ou falhar.
    async fn try_scraper_service(&self, url: &str) -> Option<ProductDraft> {
        let scraper_url = std::env::var("SCRAPER_URL").ok().filter(|s| !s.trim().is_empty())?;
        let api_key = std::env::var("SCRAPER_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var("APICASH_API_KEY").ok().filter(|s| !s.is_empty()));

        let endpoint = format!("{}/scrape", scraper_url.trim_end_matches('/'));
        let body = serde_json::json!({ "url": url });

        let mut req = self.client
            .post(&endpoint)
            .json(&body)
            .timeout(std::time::Duration::from_secs(45)); // Playwright pode demorar

        if let Some(key) = api_key {
            req = req.header("x-api-key", key);
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!(error = %e, "tiktok: scraper-service indisponível, usando fallback");
                return None;
            }
        };

        let scraped: ScraperResponse = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "tiktok: scraper-service resposta inválida");
                return None;
            }
        };

        if !scraped.ok {
            tracing::warn!("tiktok: scraper-service retornou ok=false");
            return None;
        }

        let data = scraped.data?;
        let title = data.title.filter(|s| !s.is_empty())?;

        let price = data.price.and_then(|p| {
            // Normalizar preço: "R$ 29,90" → "29.90"
            let digits: String = p.chars().filter(|c| c.is_ascii_digit() || *c == ',' || *c == '.').collect();
            let normalized = digits.replace(',', ".");
            // Se tiver ponto como separador de milhar (ex: "1.299,90" → "1299.90")
            if normalized.matches('.').count() > 1 {
                let last_dot = normalized.rfind('.').unwrap();
                Some(normalized[..last_dot].replace('.', "") + "." + &normalized[last_dot+1..])
            } else {
                Some(normalized)
            }
        });

        tracing::info!(
            title = %title,
            images = data.images.len(),
            has_price = price.is_some(),
            "tiktok: scraper-service extração completa"
        );

        // Parse do preço como Decimal
        let price_decimal = price.as_deref().and_then(|p| p.parse::<rust_decimal::Decimal>().ok());

        Some(ProductDraft {
            title,
            description: data.description.filter(|s| !s.is_empty()),
            price_suggested: price_decimal,
            photos: data.images,
            video_url: data.video_url,
            source_url: url.to_string(),
            source_platform: SourcePlatform::TikTok,
            extractor_used: "tiktok_playwright".to_string(),
            guarantee: None,
            condition: None,
            location: None,
            seller_name: data.seller_name,
            seller_rating: data.seller_rating,
            raw_attributes: serde_json::json!({
                "product_id": data.product_id,
                "price_original": data.price_original,
                "platform": "tiktok_shop",
                "scraper": "playwright",
            }),
        })
    }

    fn is_tiktok_url(url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.contains("tiktok.com") || lower.contains("vt.tiktok.com")
    }

    /// Faz GET sem seguir redirect e retorna o Location header.
    async fn fetch_redirect_location(&self, url: &str) -> Option<String> {
        // Cliente dedicado sem redirect para capturar o Location header
        let no_redirect_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;

        let resp = no_redirect_client
            .get(url)
            .header("User-Agent", MOBILE_UA)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "pt-BR,pt;q=0.9,en;q=0.8")
            .send()
            .await
            .ok()?;

        let status = resp.status().as_u16();
        if status == 301 || status == 302 || status == 307 || status == 308 {
            resp.headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Segue o redirect e baixa o HTML final (fallback).
    async fn fetch_html(&self, url: &str) -> Option<String> {
        self.client
            .get(url)
            .header("User-Agent", MOBILE_UA)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "pt-BR,pt;q=0.9")
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()
    }

    /// Extrai dados do URL de redirect (Location header).
    fn parse_redirect_url(redirect_url: &str) -> Option<TikTokData> {
        let parsed = Url::parse(redirect_url).ok()?;
        let params: std::collections::HashMap<_, _> = parsed.query_pairs().collect();

        // og_info = JSON URL-encoded com title e image
        let og_raw = params.get("og_info")?;
        let og_json: serde_json::Value = serde_json::from_str(og_raw).ok()?;

        let title = og_json["title"].as_str()?.trim().to_string();
        if title.is_empty() {
            return None;
        }

        let image = og_json["image"].as_str().map(|s| {
            // A image pode ter barras escapadas como \/ — normalizar
            s.replace("\\/", "/")
        });

        // username do vendedor disponível gratuitamente
        let seller_name = params.get("unique_id").map(|s| s.to_string());

        // ID do produto no path: /view/product/{id}
        let product_id = parsed
            .path_segments()
            .and_then(|mut segs| {
                segs.find(|s| s.chars().all(|c| c.is_ascii_digit()) && s.len() > 5)
            })
            .map(|s| s.to_string());

        // Região de origem
        let region = params.get("share_region").map(|s| s.to_string());

        Some(TikTokData {
            title,
            image,
            seller_name,
            product_id,
            region,
        })
    }

    /// Tenta extrair og:title + og:image do HTML final como fallback.
    fn parse_html_og(html: &str) -> Option<(String, Option<String>)> {
        use scraper::{Html, Selector};
        let doc = Html::parse_document(html);
        let sel = Selector::parse("meta").ok()?;

        let mut title = None;
        let mut image = None;

        for el in doc.select(&sel) {
            let prop = el.value().attr("property").or_else(|| el.value().attr("name")).unwrap_or("");
            let content = el.value().attr("content").unwrap_or("").trim();
            match prop {
                "og:title" if title.is_none() && !content.is_empty() => title = Some(content.to_string()),
                "og:image" if image.is_none() && !content.is_empty() => image = Some(content.to_string()),
                _ => {}
            }
        }

        title.map(|t| (t, image))
    }
}

struct TikTokData {
    title: String,
    image: Option<String>,
    seller_name: Option<String>,
    product_id: Option<String>,
    region: Option<String>,
}

#[async_trait]
impl Extractor for TikTokExtractor {
    fn name(&self) -> &'static str {
        "tiktok"
    }

    async fn extract(&self, url: &str, _html: &str) -> Result<Option<ProductDraft>, ImporterError> {
        if !Self::is_tiktok_url(url) {
            return Ok(None);
        }

        // ── Estratégia 0: Scraper Service (Playwright) — dados completos ──────
        if let Some(draft) = self.try_scraper_service(url).await {
            return Ok(Some(draft));
        }

        // ── Estratégia 1: Location header do redirect (melhor fonte) ──────────
        let data = if let Some(location) = self.fetch_redirect_location(url).await {
            tracing::debug!(url, redirect = %location, "tiktok: got redirect location");
            Self::parse_redirect_url(&location)
        } else {
            None
        };

        // ── Estratégia 2: OpenGraph no HTML final (fallback) ──────────────────
        let (title, image, seller_name, raw_attributes) = if let Some(d) = data {
            let raw = serde_json::json!({
                "product_id": d.product_id,
                "share_region": d.region,
                "platform": "tiktok_shop",
            });
            (d.title, d.image, d.seller_name, raw)
        } else {
            tracing::debug!(url, "tiktok: redirect failed, trying HTML fallback");
            let html = self.fetch_html(url).await;
            if let Some(ref h) = html {
                if let Some((t, img)) = Self::parse_html_og(h) {
                    (t, img, None, serde_json::json!({"platform": "tiktok_shop", "fallback": "og_html"}))
                } else {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
        };

        tracing::info!(
            title = %title,
            has_image = image.is_some(),
            seller = ?seller_name,
            "tiktok: extração concluída"
        );

        let photos: Vec<String> = image.into_iter().collect();

        Ok(Some(ProductDraft {
            title,
            description: None, // Não disponível via HTTP (requer JavaScript)
            price_suggested: None, // Não disponível via HTTP (requer JavaScript)
            photos,
            video_url: None, // MP4 protegido no TikTok
            source_url: url.to_string(),
            source_platform: SourcePlatform::TikTok,
            extractor_used: self.name().to_string(),
            guarantee: None,
            condition: None,
            location: None,
            seller_name,
            seller_rating: None,
            raw_attributes,
        }))
    }
}
