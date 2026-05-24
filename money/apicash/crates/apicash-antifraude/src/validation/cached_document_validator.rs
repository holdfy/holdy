//! Decorator that wraps any [`DocumentValidator`] with a [`DocumentCache`].
//!
//! Flow: cache hit → return immediately.
//!       cache miss → delegate to inner validator → cache definitive results → return.
//!
//! Only `Valid` and `Invalid` are cached. `Unknown` is never stored so the
//! provider is always retried on the next request.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::error::AntiFraudeError;
use crate::validation::document_cache::DocumentCache;
use crate::validation::document_validator::{DocumentStatus, DocumentType, DocumentValidator};

pub struct CachedDocumentValidator {
    inner: Arc<dyn DocumentValidator>,
    cache: Arc<dyn DocumentCache>,
    ttl: Duration,
}

impl CachedDocumentValidator {
    /// `ttl` — how long a definitive (`Valid` / `Invalid`) result is kept.
    pub fn new(
        inner: Arc<dyn DocumentValidator>,
        cache: Arc<dyn DocumentCache>,
        ttl: Duration,
    ) -> Self {
        Self { inner, cache, ttl }
    }
}

#[async_trait]
impl DocumentValidator for CachedDocumentValidator {
    async fn validate(
        &self,
        document: &str,
        doc_type: DocumentType,
    ) -> Result<DocumentStatus, AntiFraudeError> {
        if let Some(cached) = self.cache.get(document, doc_type).await {
            tracing::debug!(%document, "document_cache: hit");
            return Ok(cached);
        }

        let status = self.inner.validate(document, doc_type).await?;

        if status != DocumentStatus::Unknown {
            let _ = self.cache.set(document, doc_type, status, self.ttl).await;
        }

        Ok(status)
    }
}
