//! Application configuration loaded from environment (and optional files).

use serde::{Deserialize, Serialize};

use crate::error::ApiCashError;

/// Root configuration for APICash services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Deployment environment label (`development`, `staging`, `production`).
    pub env: String,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub stellar: StellarConfig,
    pub pulsar: PulsarConfig,
    pub whatsapp: WhatsAppConfig,
    pub antifraude: AntiFraudConfig,
    pub auth: AuthConfig,
    pub notifications: NotificationsConfig,
}

/// PostgreSQL connection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_pool")]
    pub max_connections: u32,
}

fn default_pool() -> u32 {
    10
}

/// Redis cache / session backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

/// Stellar network and Horizon / RPC endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarConfig {
    /// `testnet`, `mainnet`, `futurenet`, etc.
    pub network: String,
    pub horizon_url: String,
    pub rpc_url: String,
    pub anchor_url: Option<String>,
}

/// Apache Pulsar client settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulsarConfig {
    pub service_url: String,
    pub tenant: String,
    pub namespace: String,
}

/// WhatsApp Business / Cloud API credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    pub token: Option<String>,
    pub phone_number_id: Option<String>,
    pub business_account_id: Option<String>,
    pub webhook_verify_token: Option<String>,
}

/// Anti-fraud integrations (SEFAZ, social signals, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiFraudConfig {
    #[serde(default)]
    pub enabled: bool,
    pub sefaz_api_url: Option<String>,
    pub social_score_api_key: Option<String>,
}

/// JWT / session parameters for APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub jwt_secret: String,
}

/// Outbound notification channels (email/SMS toggles and endpoints).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    pub smtp_host: Option<String>,
    pub sms_provider_api_key: Option<String>,
}

impl AppConfig {
    /// Load `.env` if present, then merge `APICASH__*` environment variables.
    ///
    /// Nested keys use double underscore, e.g. `APICASH__DATABASE__URL`.
    pub fn load() -> Result<Self, ApiCashError> {
        let _ = dotenvy::dotenv();
        let cfg = config::Config::builder()
            .add_source(config::Environment::with_prefix("APICASH").separator("__"))
            .build()?;
        Ok(cfg.try_deserialize()?)
    }
}
