use std::{
    collections::HashMap,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct TxRecord {
    pub id: String,
    pub amount: String,
    pub pix_key: Option<String>,
    pub kind: TxKind,
    pub created_at: DateTime<Utc>,
    /// QR EMV payload — preenchido no on-ramp.
    pub pix_br_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxKind {
    OnRamp,
    OffRamp,
}

#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub gatebox_base_url: Option<String>,
    pub gatebox_api_key: Option<String>,
    pub auto_settle_ms: u64,
    pub asset_code: String,
    pub transactions: Arc<RwLock<HashMap<String, TxRecord>>>,
}

impl AppState {
    pub fn from_env() -> Self {
        let gatebox_base_url = std::env::var("GATEBOX_BASE_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let gatebox_api_key = std::env::var("GATEBOX_API_KEY")
            .ok()
            .filter(|s| !s.is_empty());

        let auto_settle_ms = std::env::var("ANCHOR_SIM_AUTO_SETTLE_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3_000u64);

        let asset_code = std::env::var("ANCHOR_SIM_ASSET_CODE")
            .unwrap_or_else(|_| "BRLx".to_string());

        if gatebox_base_url.is_some() {
            tracing::info!(
                "simulator-anchor-pix: QR real via Gatebox ({})",
                gatebox_base_url.as_deref().unwrap_or("")
            );
        } else {
            tracing::warn!("simulator-anchor-pix: GATEBOX_BASE_URL ausente — QR fake gerado localmente");
        }

        tracing::info!(auto_settle_ms, asset_code = %asset_code, "config carregada");

        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap(),
            gatebox_base_url,
            gatebox_api_key,
            auto_settle_ms,
            asset_code,
            transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
