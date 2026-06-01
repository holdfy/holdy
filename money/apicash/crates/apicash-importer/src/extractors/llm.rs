//! Extrator de fallback: envia o HTML para a OpenAI API e solicita extração estruturada.
//!
//! Requer variável de ambiente `OPENAI_API_KEY`.
//! Modelo: `OPENAI_MODEL` (padrão `gpt-4o-mini`).

use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::error::ImporterError;
use crate::types::{ProductDraft, SourcePlatform};

use super::Extractor;

pub struct LlmExtractor {
    client: Client,
    api_key: Option<String>,
}

impl LlmExtractor {
    pub fn new(client: Client) -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").ok().filter(|s| !s.is_empty());
        Self { client, api_key }
    }
}

#[async_trait]
impl Extractor for LlmExtractor {
    fn name(&self) -> &'static str {
        "llm"
    }

    async fn extract(&self, url: &str, html: &str) -> Result<Option<ProductDraft>, ImporterError> {
        let Some(api_key) = &self.api_key else {
            tracing::debug!("LlmExtractor: OPENAI_API_KEY ausente, pulando");
            return Ok(None);
        };

        // Truncate HTML to keep costs low — first 8k chars usually contain metadata.
        let html_snippet = &html[..html.len().min(8_000)];

        let prompt = format!(
            r#"Extraia informações do produto desta página HTML. Responda APENAS com JSON válido no formato:
{{"title":"...","description":"...","price_brl":"123.45","image_urls":["https://..."],"guarantee":"...","condition":"new|used|refurbished","location":"cidade, estado","seller_name":"...","seller_rating":"...","attributes":{{"chave":"valor"}}}}

Regras:
- Use null para campos ausentes.
- condition: APENAS "new", "used" ou "refurbished".
- attributes: extraia qualquer atributo extra (material, cor, tamanho, voltagem, etc.).
- Não inclua explicações fora do JSON.

URL: {url}
HTML:
{html_snippet}"#
        );

        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        let body = serde_json::json!({
            "model": model,
            "max_tokens": 512,
            "messages": [{ "role": "user", "content": prompt }]
        });

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key.as_str()))
            .json(&body)
            .send()
            .await
            .map_err(|e| ImporterError::LlmApi(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ImporterError::LlmApi(format!("HTTP {status}: {text}")));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ImporterError::LlmApi(e.to_string()))?;

        let content = json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let extracted: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| ImporterError::Serialization(e.to_string()))?;

        let title = extracted
            .get("title")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        let Some(title) = title else {
            return Ok(None);
        };

        let description = extracted
            .get("description")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        let price_suggested = extracted
            .get("price_brl")
            .and_then(|v| v.as_str())
            .and_then(|s| Decimal::from_str(s.trim()).ok());

        let photos: Vec<String> = extracted
            .get("image_urls")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        let guarantee = extracted.get("guarantee").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string);
        let condition = extracted.get("condition").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string);
        let location = extracted.get("location").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string);
        let seller_name = extracted.get("seller_name").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string);
        let seller_rating = extracted.get("seller_rating").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string);
        let raw_attributes = extracted.get("attributes").cloned().unwrap_or_else(|| serde_json::Value::Object(Default::default()));

        Ok(Some(ProductDraft {
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
        }))
    }
}
