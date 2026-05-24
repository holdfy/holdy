//! Social network URL validation (Instagram, TikTok, Facebook).
//!
//! With `SOCIAL_CHECK_ENABLED=1`: sends a HEAD request to verify the profile URL is reachable.
//! Account age remains a heuristic (no free public API provides this without OAuth).
//! Without the flag or with `mock` feature: returns realistic structured data without outbound calls.

use crate::error::AntiFraudeError;
use crate::validation::validation_result::{SocialAccountSnapshot, SocialValidationResult};
use reqwest::Client;
#[cfg(not(feature = "mock"))]
use std::time::Duration;
use tracing::instrument;
#[cfg(not(feature = "mock"))]
use tracing::{info, warn};

/// Validates profile URLs and extracts age / consistency signals.
pub struct SocialValidator {
    #[cfg(not(feature = "mock"))]
    client: Client,
    /// When true, performs a real HEAD request to verify profile reachability.
    #[cfg(not(feature = "mock"))]
    check_enabled: bool,
}

impl SocialValidator {
    #[cfg_attr(feature = "mock", allow(unused_variables))]
    pub fn new(client: Client, check_enabled: bool) -> Self {
        Self {
            #[cfg(not(feature = "mock"))]
            client,
            #[cfg(not(feature = "mock"))]
            check_enabled,
        }
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
            return Ok(mock_social(&lower, url));
        }

        #[cfg(not(feature = "mock"))]
        {
            if self.check_enabled {
                return Ok(self.head_check(&lower, url).await);
            }
            Ok(mock_social(&lower, url))
        }
    }

    #[cfg(not(feature = "mock"))]
    async fn head_check(&self, lower: &str, original: &str) -> SocialValidationResult {
        info!(url = %original, "social: HEAD check");
        let platform = detect_platform(lower).unwrap_or("unknown");
        let handle = extract_handle(lower);

        let result = self
            .client
            .head(original)
            .timeout(Duration::from_secs(5))
            .header("User-Agent", "Mozilla/5.0 (compatible; APICash/1.0)")
            .send()
            .await;

        match result {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() || status.is_redirection() {
                    // Profile URL is reachable — account likely exists.
                    SocialValidationResult {
                        url: original.to_string(),
                        snapshot: Some(SocialAccountSnapshot {
                            platform: platform.to_string(),
                            handle,
                            // Age is heuristic without OAuth — unknown → conservative 0.
                            estimated_age_months: 0,
                            name_consistent: true,
                        }),
                        error: None,
                    }
                } else {
                    warn!(url = %original, %status, "social: profile not found (non-2xx/3xx)");
                    SocialValidationResult {
                        url: original.to_string(),
                        snapshot: None,
                        error: Some(format!("profile not found (HTTP {})", status.as_u16())),
                    }
                }
            }
            Err(e) => {
                warn!(url = %original, error = %e, "social: HEAD request failed");
                SocialValidationResult {
                    url: original.to_string(),
                    snapshot: Some(SocialAccountSnapshot {
                        platform: platform.to_string(),
                        handle,
                        estimated_age_months: 0,
                        name_consistent: false,
                    }),
                    error: Some(format!("unreachable: {e}")),
                }
            }
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
