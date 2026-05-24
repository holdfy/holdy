//! Cache layer for already-validated documents (CPF / CNPJ).
//!
//! Definitive results (`Valid` / `Invalid`) are cached to avoid re-validating
//! the same document on every transaction. `Unknown` is never cached so the
//! provider is retried on the next request.
//!
//! Implementations:
//!   - [`InMemoryDocumentCache`]: in-process, lost on restart. Good for dev and tests.
//!   - [`PostgresDocumentCache`]: persisted. Requires table `document_validation_cache`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::error::AntiFraudeError;
use crate::validation::document_validator::{DocumentStatus, DocumentType};

#[async_trait]
pub trait DocumentCache: Send + Sync {
    async fn get(&self, document: &str, doc_type: DocumentType) -> Option<DocumentStatus>;
    async fn set(
        &self,
        document: &str,
        doc_type: DocumentType,
        status: DocumentStatus,
        ttl: Duration,
    ) -> Result<(), AntiFraudeError>;
}

// ─── In-memory ────────────────────────────────────────────────────────────────

struct Entry {
    status: DocumentStatus,
    expires_at: Instant,
}

pub struct InMemoryDocumentCache {
    store: RwLock<HashMap<(String, &'static str), Entry>>,
}

impl InMemoryDocumentCache {
    pub fn new() -> Self {
        Self { store: RwLock::new(HashMap::new()) }
    }
}

impl Default for InMemoryDocumentCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentCache for InMemoryDocumentCache {
    async fn get(&self, document: &str, doc_type: DocumentType) -> Option<DocumentStatus> {
        let key = (document.to_string(), doc_type.as_str());
        let store = self.store.read().await;
        store.get(&key).and_then(|e| {
            if Instant::now() < e.expires_at {
                Some(e.status)
            } else {
                None
            }
        })
    }

    async fn set(
        &self,
        document: &str,
        doc_type: DocumentType,
        status: DocumentStatus,
        ttl: Duration,
    ) -> Result<(), AntiFraudeError> {
        let key = (document.to_string(), doc_type.as_str());
        let entry = Entry { status, expires_at: Instant::now() + ttl };
        self.store.write().await.insert(key, entry);
        Ok(())
    }
}

// ─── Postgres ─────────────────────────────────────────────────────────────────

/// Persistent document validation cache backed by Postgres.
///
/// Requires migration:
/// ```sql
/// CREATE TABLE IF NOT EXISTS document_validation_cache (
///     document   TEXT        NOT NULL,
///     doc_type   TEXT        NOT NULL,
///     status     TEXT        NOT NULL,
///     expires_at TIMESTAMPTZ NOT NULL,
///     PRIMARY KEY (document, doc_type)
/// );
/// ```
pub struct PostgresDocumentCache {
    pool: sqlx::PgPool,
}

impl PostgresDocumentCache {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DocumentCache for PostgresDocumentCache {
    async fn get(&self, document: &str, doc_type: DocumentType) -> Option<DocumentStatus> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT status FROM document_validation_cache \
             WHERE document = $1 AND doc_type = $2 AND expires_at > NOW()",
        )
        .bind(document)
        .bind(doc_type.as_str())
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten();

        row.and_then(|(s,)| match s.as_str() {
            "valid" => Some(DocumentStatus::Valid),
            "invalid" => Some(DocumentStatus::Invalid),
            _ => None,
        })
    }

    async fn set(
        &self,
        document: &str,
        doc_type: DocumentType,
        status: DocumentStatus,
        ttl: Duration,
    ) -> Result<(), AntiFraudeError> {
        let status_str = match status {
            DocumentStatus::Valid => "valid",
            DocumentStatus::Invalid => "invalid",
            DocumentStatus::Unknown => return Ok(()), // never persist Unknown
        };
        let secs = ttl.as_secs() as i64;
        sqlx::query(
            "INSERT INTO document_validation_cache (document, doc_type, status, expires_at) \
             VALUES ($1, $2, $3, NOW() + ($4 || ' seconds')::INTERVAL) \
             ON CONFLICT (document, doc_type) DO UPDATE \
             SET status = EXCLUDED.status, expires_at = EXCLUDED.expires_at",
        )
        .bind(document)
        .bind(doc_type.as_str())
        .bind(status_str)
        .bind(secs.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AntiFraudeError::Internal(e.to_string()))?;
        Ok(())
    }
}
