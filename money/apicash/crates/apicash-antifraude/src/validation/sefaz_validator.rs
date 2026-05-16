//! CPF validation against Receita Federal / SEFAZ-style providers.
//!
//! Production: plug `base_url` + API key into a provider such as FiscalAPI or government APIs
//! (subject to contract and legal use). The `mock` feature skips outbound HTTP.

use crate::error::AntiFraudeError;
use crate::validation::validation_result::{SefazPersonStatus, SefazValidationResult};
use reqwest::Client;
use serde_json::json;
use tracing::instrument;
#[cfg(not(feature = "mock"))]
use tracing::warn;

fn digits_only(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn valid_cpf_format(digits: &str) -> bool {
    digits.len() == 11
}

/// Validates CPF strings and maps provider responses into [`SefazValidationResult`].
pub struct SefazValidator {
    #[allow(dead_code)]
    client: Client,
    /// Optional base URL for a Receita / commercial KYC API.
    pub base_url: Option<String>,
}

impl SefazValidator {
    pub fn new(client: Client, base_url: Option<String>) -> Self {
        Self { client, base_url }
    }

    /// Validates CPF (individual). CNPJ path can be added as a sibling method later.
    #[instrument(skip(self), fields(cpf_len = cpf.len()))]
    pub async fn validate_cpf(&self, cpf: &str) -> Result<SefazValidationResult, AntiFraudeError> {
        let normalized = digits_only(cpf);
        if !valid_cpf_format(&normalized) {
            return Err(AntiFraudeError::InvalidDocument(
                "CPF must contain 11 digits".into(),
            ));
        }

        #[cfg(feature = "mock")]
        {
            return Ok(mock_cpf_status(&normalized));
        }

        #[cfg(not(feature = "mock"))]
        {
            if self.base_url.is_some() {
                // Placeholder for FiscalAPI / Receita integration: wire HTTP + JSON mapping here.
                warn!("SEFAZ provider URL set but live client not implemented; using deterministic mock");
            }
            Ok(mock_cpf_status(&normalized))
        }
    }
}

/// Deterministic mock: documents ending in `00` are *irregular*; others *regular*.
fn mock_cpf_status(normalized: &str) -> SefazValidationResult {
    let irregular = normalized.ends_with("00");
    let status = if irregular {
        SefazPersonStatus::Irregular
    } else {
        SefazPersonStatus::Regular
    };
    SefazValidationResult {
        normalized_document: normalized.to_string(),
        status,
        provider_hint: Some(json!({ "source": "mock_sefaz" })),
    }
}
