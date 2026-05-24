//! HTTP-backed document validation for CPF/CNPJ via an external provider
//! (FiscalAPI, Serpro reseller, or any provider matching the generic contract).
//!
//! Provider contract:
//!   GET {base_url}/v1/{doc_type}/{digits}
//!   Authorization: Bearer {api_token}
//!   200 → { "situacao": "Regular" | <anything else> }
//!
//! On network errors or non-2xx responses the result is `Unknown` — the flow
//! continues and is never blocked solely due to provider unavailability.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use tracing::{info, warn};

use crate::error::AntiFraudeError;
use crate::validation::document_validator::{DocumentStatus, DocumentType, DocumentValidator};

pub struct HttpDocumentValidator {
    client: Client,
    base_url: String,
    api_token: Option<String>,
}

impl HttpDocumentValidator {
    pub fn new(client: Client, base_url: String, api_token: Option<String>) -> Self {
        Self { client, base_url, api_token }
    }

    async fn fetch_status(
        &self,
        digits: &str,
        doc_type: DocumentType,
    ) -> Result<DocumentStatus, AntiFraudeError> {
        let base = self.base_url.trim_end_matches('/');
        let url = format!("{}/v1/{}/{}", base, doc_type.as_str(), digits);
        info!(doc_type = doc_type.as_str(), "document_validator: querying HTTP provider");

        let mut req = self.client.get(&url).timeout(Duration::from_secs(8));
        if let Some(ref token) = self.api_token {
            req = req.bearer_auth(token);
        }

        let response = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, "document_validator: provider unreachable — Unknown");
                return Ok(DocumentStatus::Unknown);
            }
        };

        if !response.status().is_success() {
            warn!(code = %response.status(), "document_validator: non-2xx — Unknown");
            return Ok(DocumentStatus::Unknown);
        }

        let body: serde_json::Value = match response.json().await {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "document_validator: parse error — Unknown");
                return Ok(DocumentStatus::Unknown);
            }
        };

        let situacao = body
            .get("situacao")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let status = match situacao.as_str() {
            "regular" => DocumentStatus::Valid,
            "" => DocumentStatus::Unknown,
            _ => DocumentStatus::Invalid,
        };

        // Preserve provider payload for callers that want to log it.
        tracing::debug!(status_str = %situacao, "document_validator: provider response");

        Ok(status)
    }
}

#[async_trait]
impl DocumentValidator for HttpDocumentValidator {
    async fn validate(
        &self,
        document: &str,
        doc_type: DocumentType,
    ) -> Result<DocumentStatus, AntiFraudeError> {
        let digits: String = document.chars().filter(|c| c.is_ascii_digit()).collect();
        let expected_len = match doc_type {
            DocumentType::Cpf => 11,
            DocumentType::Cnpj => 14,
        };
        if digits.len() != expected_len {
            return Err(AntiFraudeError::InvalidDocument(format!(
                "{} must have {} digits",
                doc_type.as_str().to_uppercase(),
                expected_len
            )));
        }
        self.fetch_status(&digits, doc_type).await
    }
}
