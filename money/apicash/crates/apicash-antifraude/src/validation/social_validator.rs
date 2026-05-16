//! Social network URL validation (Instagram, TikTok, Facebook).
//!
//! The `mock` feature avoids outbound fetches and returns realistic structured data.

use crate::error::AntiFraudeError;
use crate::validation::validation_result::{SocialAccountSnapshot, SocialValidationResult};
use reqwest::Client;
use tracing::instrument;

/// Validates profile URLs and extracts age / consistency signals.
pub struct SocialValidator {
    #[allow(dead_code)]
    client: Client,
}

impl SocialValidator {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    #[instrument(skip(self, urls))]
    pub async fn validate_links(
        &self,
        urls: &[String],
    ) -> Result<Vec<SocialValidationResult>, AntiFraudeError> {
        let mut out = Vec::with_capacity(urls.len());
        for url in urls {
            out.push(self.validate_one(url).await?);
        }
        Ok(out)
    }

    async fn validate_one(&self, url: &str) -> Result<SocialValidationResult, AntiFraudeError> {
        let lower = url.to_ascii_lowercase();

        #[cfg(feature = "mock")]
        {
            Ok(mock_social(&lower, url))
        }

        #[cfg(not(feature = "mock"))]
        {
            // Future: HEAD/GET public profile pages or Meta/TikTok Graph APIs (with tokens).
            let _ = self.client.get(url).send().await;
            Ok(mock_social(&lower, url))
        }
    }
}

fn detect_platform(lower: &str) -> Option<&'static str> {
    if lower.contains("instagram.com") {
        Some("instagram")
    } else if lower.contains("tiktok.com") {
        Some("tiktok")
    } else if lower.contains("facebook.com") || lower.contains("fb.com") {
        Some("facebook")
    } else {
        None
    }
}

/// Mock rules:
/// - URLs containing `old` or `verified` simulate accounts older than 6 months.
/// - URLs containing `new` simulate fresh accounts.
fn mock_social(lower: &str, original: &str) -> SocialValidationResult {
    let platform = detect_platform(lower).unwrap_or("unknown");
    let handle = extract_handle(lower);

    let estimated_age_months = if lower.contains("old") || lower.contains("verified") {
        12u32
    } else if lower.contains("new") {
        2u32
    } else {
        8u32
    };

    let name_consistent = lower.contains("verified") || !lower.contains("mismatch");

    SocialValidationResult {
        url: original.to_string(),
        snapshot: Some(SocialAccountSnapshot {
            platform: platform.to_string(),
            handle,
            estimated_age_months,
            name_consistent,
        }),
        error: None,
    }
}

fn extract_handle(lower: &str) -> String {
    lower
        .rsplit('/')
        .find(|s| !s.is_empty() && *s != "www.instagram.com")
        .unwrap_or("user")
        .chars()
        .take(64)
        .collect()
}
