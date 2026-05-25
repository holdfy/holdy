//! ImporterService: fetches the URL and runs extractors in cascade.

use reqwest::Client;
use url::Url;

use crate::error::ImporterError;
use crate::extractors::{
    Extractor, JsonLdExtractor, LlmExtractor, MercadoLivreExtractor, OpenGraphExtractor,
};
use crate::types::ProductDraft;

/// Orchestrates the extractor pipeline.
pub struct ImporterService {
    client: Client,
    extractors: Vec<Box<dyn Extractor>>,
}

impl ImporterService {
    pub fn new() -> Self {
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

        Self { client, extractors }
    }

    /// Import a product from `url`.
    ///
    /// Validates the URL, fetches HTML once, then tries each extractor in order.
    /// Returns the first `ProductDraft` produced.
    pub async fn import(&self, url: &str) -> Result<ProductDraft, ImporterError> {
        // Validate URL and block internal IPs.
        let parsed = Url::parse(url).map_err(|_| ImporterError::InvalidUrl(url.to_string()))?;
        let scheme = parsed.scheme();
        if scheme != "https" && scheme != "http" {
            return Err(ImporterError::InvalidUrl(format!(
                "scheme '{scheme}' não suportado"
            )));
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
                Ok(Some(draft)) => {
                    tracing::info!(extractor = extractor.name(), url, "importer: extracted");
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
}

impl Default for ImporterService {
    fn default() -> Self {
        Self::new()
    }
}
