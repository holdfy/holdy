//! Extrai `schema.org/Product` de blocos JSON-LD embutidos no HTML.
//! Cobre OLX, Shopee e a maioria dos e-commerces.

use async_trait::async_trait;
use rust_decimal::Decimal;
use scraper::{Html, Selector};
use std::str::FromStr;

use crate::error::ImporterError;
use crate::types::{ProductDraft, SourcePlatform};

use super::Extractor;

pub struct JsonLdExtractor;

#[async_trait]
impl Extractor for JsonLdExtractor {
    fn name(&self) -> &'static str {
        "json_ld"
    }

    async fn extract(&self, url: &str, html: &str) -> Result<Option<ProductDraft>, ImporterError> {
        let document = Html::parse_document(html);
        let selector = Selector::parse(r#"script[type="application/ld+json"]"#)
            .map_err(|e| ImporterError::FetchFailed(e.to_string()))?;

        for element in document.select(&selector) {
            let text = element.text().collect::<String>();
            let Ok(json): Result<serde_json::Value, _> = serde_json::from_str(&text) else {
                continue;
            };

            // Handle both single object and @graph array.
            let candidates: Vec<&serde_json::Value> = if let Some(graph) = json.get("@graph") {
                graph.as_array().map(|a| a.iter().collect()).unwrap_or_default()
            } else {
                vec![&json]
            };

            for node in candidates {
                let type_val = node.get("@type").and_then(|v| v.as_str()).unwrap_or("");
                if !type_val.contains("Product") {
                    continue;
                }

                let title = node
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let Some(title) = title else { continue };

                let description = node
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);

                let price_suggested = extract_price(node);

                let photos = extract_images(node);

                return Ok(Some(ProductDraft {
                    title,
                    description,
                    price_suggested,
                    photos,
                    source_url: url.to_string(),
                    source_platform: SourcePlatform::detect(url),
                    extractor_used: self.name().to_string(),
                }));
            }
        }

        Ok(None)
    }
}

fn extract_price(node: &serde_json::Value) -> Option<Decimal> {
    // Try `offers.price`, `offers[0].price`, `price`
    let offers = node.get("offers");
    let price_str = offers
        .and_then(|o| {
            if o.is_array() {
                o.as_array()?.first()?.get("price")?.as_str().map(str::to_string)
            } else {
                o.get("price")?.as_str().map(str::to_string)
            }
        })
        .or_else(|| node.get("price")?.as_str().map(str::to_string));

    price_str
        .as_deref()
        .and_then(|s| Decimal::from_str(s.trim().trim_start_matches('R').trim_start_matches('$').trim()).ok())
}

fn extract_images(node: &serde_json::Value) -> Vec<String> {
    let mut urls = vec![];

    if let Some(img) = node.get("image") {
        match img {
            serde_json::Value::String(s) => urls.push(s.clone()),
            serde_json::Value::Array(arr) => {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        urls.push(s.to_string());
                    } else if let Some(url) = v.get("url").and_then(|u| u.as_str()) {
                        urls.push(url.to_string());
                    }
                }
            }
            serde_json::Value::Object(obj) => {
                if let Some(url) = obj.get("url").and_then(|u| u.as_str()) {
                    urls.push(url.to_string());
                }
            }
            _ => {}
        }
    }

    urls
}
