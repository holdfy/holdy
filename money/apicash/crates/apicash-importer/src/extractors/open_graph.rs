//! Extrai `og:title`, `og:image`, `og:description` das meta tags.
//! Cobre Instagram, Facebook, TikTok e qualquer site com OpenGraph.

use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::error::ImporterError;
use crate::types::{ProductDraft, SourcePlatform};

use super::Extractor;

pub struct OpenGraphExtractor;

#[async_trait]
impl Extractor for OpenGraphExtractor {
    fn name(&self) -> &'static str {
        "open_graph"
    }

    async fn extract(&self, url: &str, html: &str) -> Result<Option<ProductDraft>, ImporterError> {
        let document = Html::parse_document(html);
        let meta_sel = Selector::parse("meta")
            .map_err(|e| ImporterError::FetchFailed(e.to_string()))?;

        let mut title = None::<String>;
        let mut description = None::<String>;
        let mut image = None::<String>;

        for el in document.select(&meta_sel) {
            let property = el
                .value()
                .attr("property")
                .or_else(|| el.value().attr("name"))
                .unwrap_or("");
            let content = el.value().attr("content").unwrap_or("").trim();

            match property {
                "og:title" if title.is_none() => title = Some(content.to_string()),
                "og:description" if description.is_none() => description = Some(content.to_string()),
                "og:image" if image.is_none() => image = Some(content.to_string()),
                _ => {}
            }
        }

        let Some(title) = title.filter(|s| !s.is_empty()) else {
            return Ok(None);
        };

        let photos = image.into_iter().collect();

        Ok(Some(ProductDraft {
            title,
            description: description.filter(|s| !s.is_empty()),
            price_suggested: None,
            photos,
            source_url: url.to_string(),
            source_platform: SourcePlatform::detect(url),
            extractor_used: self.name().to_string(),
            guarantee: None,
            condition: None,
            location: None,
            seller_name: None,
            seller_rating: None,
            raw_attributes: serde_json::Value::Object(Default::default()),
        }))
    }
}
