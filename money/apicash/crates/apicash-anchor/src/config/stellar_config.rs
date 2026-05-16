//! Stellar / Anchor configuration loaded **only** from environment variables (never hardcoded secrets).
//!
//! **BRLx** (or the configured [`StellarConfig::asset_code`]) represents tokenized BRL on Stellar; real fiat
//! moves through the Anchor’s banking rails (e.g. PIX), while the on-ledger balance is the token.

use serde::{Deserialize, Serialize};

use crate::errors::AnchorError;

/// Which public Stellar network to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StellarNetwork {
    Testnet,
    Mainnet,
}

/// Connection and asset parameters for Horizon, Anchor, and operational signing.
#[derive(Debug, Clone)]
pub struct StellarConfig {
    pub network: StellarNetwork,
    /// Anchor SEP server base URL (SEP-6 / SEP-24).
    pub anchor_url: String,
    /// Issued asset code on Stellar (e.g. `BRLx`).
    pub asset_code: String,
    /// Horizon REST base URL.
    pub horizon_url: String,
    /// Stellar secret key for operational accounts — **must** stay in env / KMS; never log.
    pub secret_key: String,
}

impl StellarConfig {
    /// Load from process environment (optionally after [`dotenvy::dotenv()`]).
    ///
    /// Required vars:
    /// - `APICASH_STELLAR_NETWORK` — `testnet` | `mainnet`
    /// - `APICASH_STELLAR_ANCHOR_URL`
    /// - `APICASH_STELLAR_ASSET_CODE`
    /// - `APICASH_STELLAR_HORIZON_URL`
    /// - `APICASH_STELLAR_SECRET_KEY`
    pub fn from_env() -> Result<Self, AnchorError> {
        let network = match std::env::var("APICASH_STELLAR_NETWORK")
            .map_err(|_| AnchorError::Config("APICASH_STELLAR_NETWORK missing".into()))?
            .to_ascii_lowercase()
            .as_str()
        {
            "testnet" => StellarNetwork::Testnet,
            "mainnet" | "pubnet" => StellarNetwork::Mainnet,
            other => {
                return Err(AnchorError::Config(format!(
                    "unknown APICASH_STELLAR_NETWORK: {other}"
                )));
            }
        };

        Ok(Self {
            network,
            anchor_url: read_env("APICASH_STELLAR_ANCHOR_URL")?,
            asset_code: read_env("APICASH_STELLAR_ASSET_CODE")?,
            horizon_url: read_env("APICASH_STELLAR_HORIZON_URL")?,
            secret_key: read_env("APICASH_STELLAR_SECRET_KEY")?,
        })
    }
}

fn read_env(key: &'static str) -> Result<String, AnchorError> {
    std::env::var(key).map_err(|_| AnchorError::Config(format!("{key} is not set")))
}
