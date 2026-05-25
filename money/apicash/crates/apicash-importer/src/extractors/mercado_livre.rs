//! Extrator via API oficial do MercadoLivre.
//! Detecta URLs `mercadolivre.com.br/…-MLB{id}…` e consulta
//! `api.mercadolibre.com/items/{id}`.

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::error::ImporterError;
use crate::types::{ProductDraft, SourcePlatform};

use super::Extractor;

pub struct MercadoLivreExtractor {
    client: Client,
}

impl MercadoLivreExtractor {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn extract_item_id(url: &str) -> Option<String> {
        // e.g. https://www.mercadolivre.com.br/...-MLB123456789-...
        let re_patterns = ["-MLB", "_MLB"];
        for pat in &re_patterns {
            if let Some(pos) = url.find(pat) {
                let rest = &url[pos + pat.len()..];
                let id: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if !id.is_empty() {
                    return Some(format!("MLB{id}"));
                }
            }
        }
        None
    }
}

#[async_trait]
impl Extractor for MercadoLivreExtractor {
    fn name(&self) -> &'static str {
        "mercado_livre"
    }

    async fn extract(&self, url: &str, _html: &str) -> Result<Option<ProductDraft>, ImporterError> {
        let lower = url.to_lowercase();
        if !lower.contains("mercadolivre") && !lower.contains("mercadolibre") {
            return Ok(None);
        }

        let item_id = match Self::extract_item_id(url) {
            Some(id) => id,
            None => return Ok(None),
        };

        let api_url = format!("https://api.mercadolibre.com/items/{item_id}");
        let resp = self
            .client
            .get(&api_url)
            .send()
            .await
            .map_err(|e| ImporterError::MercadoLivreApi(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ImporterError::MercadoLivreApi(format!(
                "HTTP {}",
                resp.status()
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ImporterError::MercadoLivreApi(e.to_string()))?;

        let title = json
            .get("title")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| ImporterError::MercadoLivreApi("campo 'title' ausente".into()))?;

        let price_suggested = json
            .get("price")
            .and_then(|v| v.as_f64())
            .and_then(|f| Decimal::from_str(&f.to_string()).ok());

        let photos: Vec<String> = json
            .get("pictures")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| p.get("secure_url").and_then(|u| u.as_str()))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        let description = json
            .get("description")
            .and_then(|v| v.get("plain_text"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        Ok(Some(ProductDraft {
            title,
            description,
            price_suggested,
            photos,
            source_url: url.to_string(),
            source_platform: SourcePlatform::MercadoLivre,
            extractor_used: self.name().to_string(),
        }))
    }
}
