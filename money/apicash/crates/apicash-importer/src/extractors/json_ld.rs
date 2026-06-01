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

                let guarantee = extract_guarantee(node);
                let condition = extract_condition(node);
                let location = extract_location(node);
                let seller_name = extract_seller_name(node);
                let seller_rating = extract_seller_rating(node);
                let raw_attributes = extract_raw_attributes(node);

                return Ok(Some(ProductDraft {
                    title,
                    description,
                    price_suggested,
                    photos,
                    source_url: url.to_string(),
                    source_platform: SourcePlatform::detect(url),
                    extractor_used: self.name().to_string(),
                    guarantee,
                    condition,
                    location,
                    seller_name,
                    seller_rating,
                    video_url: None,
                    raw_attributes,
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

fn extract_guarantee(node: &serde_json::Value) -> Option<String> {
    // schema.org: `warranty` or additionalProperty named "Garantia"
    node.get("warranty")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| extract_additional_property(node, &["Garantia", "Warranty", "garantia"]))
}

fn extract_condition(node: &serde_json::Value) -> Option<String> {
    // schema.org: `itemCondition` — "NewCondition", "UsedCondition", "RefurbishedCondition"
    node.get("itemCondition")
        .and_then(|v| v.as_str())
        .map(|s| {
            let lower = s.to_lowercase();
            if lower.contains("new") { "new".to_string() }
            else if lower.contains("used") { "used".to_string() }
            else if lower.contains("refurb") { "refurbished".to_string() }
            else { lower }
        })
        .or_else(|| extract_additional_property(node, &["Condição", "Condition"]))
}

fn extract_location(node: &serde_json::Value) -> Option<String> {
    // `offers.availableAtOrFrom.address` or `offers.availableAtOrFrom.name`
    let offers = node.get("offers")?;
    let place = if offers.is_array() {
        offers.as_array()?.first()?.get("availableAtOrFrom")?
    } else {
        offers.get("availableAtOrFrom")?
    };
    place.get("address")
        .and_then(|a| a.as_str().map(str::to_string).or_else(|| a.get("addressLocality").and_then(|v| v.as_str()).map(str::to_string)))
        .or_else(|| place.as_str().map(str::to_string))
}

fn extract_seller_name(node: &serde_json::Value) -> Option<String> {
    // `offers.seller.name` or `brand.name`
    let from_offers = || -> Option<String> {
        let offers = node.get("offers")?;
        let o = if offers.is_array() { offers.as_array()?.first()? } else { offers };
        o.get("seller")?.get("name")?.as_str().map(str::to_string)
    };
    from_offers().or_else(|| node.get("brand")?.get("name")?.as_str().map(str::to_string))
}

fn extract_seller_rating(node: &serde_json::Value) -> Option<String> {
    node.get("aggregateRating")
        .and_then(|r| r.get("ratingValue"))
        .and_then(|v| v.as_str().map(str::to_string).or_else(|| v.as_f64().map(|f| f.to_string())))
}

fn extract_additional_property(node: &serde_json::Value, names: &[&str]) -> Option<String> {
    let props = node.get("additionalProperty")?.as_array()?;
    for prop in props {
        let name = prop.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if names.iter().any(|n| name.eq_ignore_ascii_case(n)) {
            return prop.get("value").and_then(|v| v.as_str()).map(str::to_string);
        }
    }
    None
}

fn extract_raw_attributes(node: &serde_json::Value) -> serde_json::Value {
    // Collect all `additionalProperty` as {"name": value} map
    let Some(props) = node.get("additionalProperty").and_then(|v| v.as_array()) else {
        return serde_json::Value::Object(Default::default());
    };
    let mut map = serde_json::Map::new();
    for prop in props {
        if let (Some(name), Some(value)) = (
            prop.get("name").and_then(|v| v.as_str()),
            prop.get("value"),
        ) {
            map.insert(name.to_string(), value.clone());
        }
    }
    serde_json::Value::Object(map)
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
