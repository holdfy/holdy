//! Provedor de rastreio simulado — backend LogisticaHoldFy (`apprastreio/backend`).
//!
//! Activado quando `APICASH_TRACKING_MODE=simulated` (ou `APICASH_TRACKING_SIMULATED=1`).
//! URL base: `APICASH_TRACKING_SIMULATOR_URL` (defeito: `http://{MONEY_LAN_HOST}:8092`).

use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use tracing::debug;

use crate::error::LogisticsError;
use crate::tracking::circuit_breaker::CircuitBreaker;
use crate::types::TrackingInfo;

use super::TrackingProvider;

pub struct HoldfySimulatorProvider {
    base_url: String,
    client: Client,
    pub breaker: Arc<CircuitBreaker>,
}

impl HoldfySimulatorProvider {
    pub fn from_env() -> Option<Self> {
        if !is_tracking_simulated() {
            return None;
        }
        let base_url = simulator_base_url();
        Some(Self {
            base_url,
            client: Client::new(),
            breaker: CircuitBreaker::new("holdfy_simulator", 3, 60),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// `true` quando o rastreio deve usar apenas o simulador LogisticaHoldFy.
pub fn is_tracking_simulated() -> bool {
    match std::env::var("APICASH_TRACKING_MODE")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "simulated" | "simulator" | "mock" | "holdfy" => true,
        "live" | "production" | "real" => false,
        _ => std::env::var("APICASH_TRACKING_SIMULATED")
            .map(|v| v.trim() == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
    }
}

fn simulator_base_url() -> String {
    std::env::var("APICASH_TRACKING_SIMULATOR_URL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let host = std::env::var("MONEY_LAN_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
            let port = std::env::var("LOGISTICA_HTTP_PORT").unwrap_or_else(|_| "8092".to_string());
            format!("http://{}:{port}", host.trim())
        })
        .trim_end_matches('/')
        .to_string()
}

#[async_trait]
impl TrackingProvider for HoldfySimulatorProvider {
    fn name(&self) -> &'static str {
        "logistica_holdfy"
    }

    async fn track(&self, code: &str) -> Result<TrackingInfo, LogisticsError> {
        let url = format!(
            "{}/logistics/tracking/{}",
            self.base_url,
            urlencoding_encode(code)
        );

        debug!(code = %code, url = %url, "holdfy_simulator: consultando rastreio");

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| LogisticsError::RequestFailed(format!("holdfy_simulator: {e}")))?;

        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(LogisticsError::TrackingNotFound(code.to_string()));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(LogisticsError::ApiError(format!(
                "holdfy_simulator HTTP {status}: {body}"
            )));
        }

        resp.json::<TrackingInfo>()
            .await
            .map_err(|e| LogisticsError::RequestFailed(format!("holdfy_simulator parse: {e}")))
    }
}

fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                vec![c]
            } else {
                format!("%{:02X}", c as u32).chars().collect()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulated_mode_from_env_var() {
        std::env::set_var("APICASH_TRACKING_MODE", "simulated");
        std::env::remove_var("APICASH_TRACKING_SIMULATED");
        assert!(is_tracking_simulated());
        std::env::remove_var("APICASH_TRACKING_MODE");
    }

    #[test]
    fn live_mode_disables_simulator() {
        std::env::set_var("APICASH_TRACKING_MODE", "live");
        std::env::set_var("APICASH_TRACKING_SIMULATED", "1");
        assert!(!is_tracking_simulated());
        std::env::remove_var("APICASH_TRACKING_MODE");
        std::env::remove_var("APICASH_TRACKING_SIMULATED");
    }
}
