//! Pluggable extractors tried in cascade order.

use async_trait::async_trait;

use crate::error::ImporterError;
use crate::types::ProductDraft;

pub mod json_ld;
pub mod llm;
pub mod mercado_livre;
pub mod open_graph;
pub mod tiktok;

pub use json_ld::JsonLdExtractor;
pub use llm::LlmExtractor;
pub use mercado_livre::MercadoLivreExtractor;
pub use open_graph::OpenGraphExtractor;
pub use tiktok::TikTokExtractor;

/// A single extraction strategy.
#[async_trait]
pub trait Extractor: Send + Sync {
    fn name(&self) -> &'static str;

    /// Returns `Ok(Some(draft))` on success, `Ok(None)` if this extractor
    /// cannot handle the URL / found no data (pipeline continues to next).
    async fn extract(&self, url: &str, html: &str) -> Result<Option<ProductDraft>, ImporterError>;
}
