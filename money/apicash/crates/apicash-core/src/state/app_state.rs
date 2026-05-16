//! Shared application state: domain services + order persistence.

use std::sync::Arc;

use apicash_anchor::{AnchorService, StellarConfig, StellarNetwork};
use apicash_antifraude::{
    AntiFraudeService, InMemoryScoreRepository, PostgresScoreRepository, ScoreRepository,
    SefazValidator, SocialValidator,
};
use apicash_auth::{AuthConfig, AuthService};
use apicash_custody::{
    CustodyRepository, CustodyService, InMemoryCustodyRepository, PostgresCustodyRepository,
    YieldCalculator,
};
use apicash_shared::Order;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

use super::order_repository::{InMemoryOrderRepository, OrderRepository, PostgresOrderRepository};

/// Snapshot stored after a successful create-order pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredOrder {
    pub order: Order,
    pub custody_id: Option<Uuid>,
    pub anchor_tx_hash: Option<String>,
    pub fiat_rail: String,
    pub gateway_in_tx_id: Option<String>,
    pub funding_reference: Option<String>,
    pub pix_br_code: Option<String>,
    /// Human-readable funding steps when applicable (neutral wording — never SEP-branding).
    pub funding_instruction: Option<String>,
    pub risk_score: u32,
    pub risk_decision: String,
    /// Descrição livre do pedido (WhatsApp / clientes).
    pub description: Option<String>,
    /// Hash da transação de off-ramp (mock), quando já executado.
    pub off_ramp_tx_hash: Option<String>,
    /// Transferência BRLx → contrato Soroban (pós on-ramp).
    pub brlx_escrow_transfer_tx_hash: Option<String>,
    pub soroban_escrow_contract_id: Option<String>,
    pub soroban_lock_tx_hash: Option<String>,
    /// `"mock"` ou `"soroban"` conforme env / bridge.
    pub soroban_mode: String,
}

/// Application state injected into Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub antifraude: Arc<AntiFraudeService>,
    pub custody: Arc<CustodyService>,
    pub anchor: Arc<AnchorService>,
    pub orders: Arc<dyn OrderRepository>,
}

impl AppState {
    /// Build services from environment (with dev fallbacks when Stellar env is incomplete).
    pub fn new() -> Self {
        Self::with_auth_config(AuthConfig::from_env())
    }

    /// Arranque assíncrono: Postgres só com `APICASH_CUSTODY_PG=1` / `APICASH_ORDERS_PG=1` e `DATABASE_URL`
    /// (após `sqlx migrate run`). Caso contrário mantém memória — evita falhar se `.env` tiver URL mas o Postgres não estiver up.
    pub async fn connect_from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let want_custody_pg = env_enabled("APICASH_CUSTODY_PG");
        let want_orders_pg = env_enabled("APICASH_ORDERS_PG");
        let want_scores_pg = env_enabled("APICASH_SCORES_PG");
        let pool = if want_custody_pg || want_orders_pg || want_scores_pg {
            let Some(url) = std::env::var("DATABASE_URL")
                .ok()
                .filter(|s| !s.trim().is_empty())
            else {
                return Err(
                    "APICASH_CUSTODY_PG/APICASH_ORDERS_PG/APICASH_SCORES_PG requires DATABASE_URL (postgresql://...)"
                        .into(),
                );
            };
            Some(
                PgPoolOptions::new()
                    .max_connections(10)
                    .connect(url.trim())
                    .await?,
            )
        } else {
            None
        };

        let custody_repo: Arc<dyn CustodyRepository> = {
            if want_custody_pg {
                tracing::info!("custody: Postgres (APICASH_CUSTODY_PG + DATABASE_URL)");
                Arc::new(PostgresCustodyRepository::new(pool.clone().expect("pool")))
            } else {
                tracing::info!(
                    "custody: in-memory (set APICASH_CUSTODY_PG=1 + DATABASE_URL for Postgres)"
                );
                Arc::new(InMemoryCustodyRepository::new())
            }
        };
        let orders: Arc<dyn OrderRepository> = if want_orders_pg {
            tracing::info!("orders: Postgres (APICASH_ORDERS_PG + DATABASE_URL)");
            Arc::new(PostgresOrderRepository::new(pool.clone().expect("pool")))
        } else {
            tracing::warn!(
                "orders: in-memory (set APICASH_ORDERS_PG=1 + DATABASE_URL for Postgres)"
            );
            Arc::new(InMemoryOrderRepository::new())
        };
        let score_repo: Arc<dyn ScoreRepository> = if want_scores_pg {
            tracing::info!("scores: Postgres (APICASH_SCORES_PG + DATABASE_URL)");
            Arc::new(PostgresScoreRepository::new(pool.clone().expect("pool")))
        } else {
            tracing::info!(
                "scores: in-memory (set APICASH_SCORES_PG=1 + DATABASE_URL for Postgres)"
            );
            Arc::new(InMemoryScoreRepository::new())
        };
        let state =
            Self::with_auth_config_repos(AuthConfig::from_env(), custody_repo, orders, score_repo);
        Ok(state)
    }

    /// Permite testes com [`AuthConfig::local_dev_open`] sem depender de env.
    pub fn with_auth_config(auth_cfg: AuthConfig) -> Self {
        Self::with_auth_config_repos(
            auth_cfg,
            Arc::new(InMemoryCustodyRepository::new()),
            Arc::new(InMemoryOrderRepository::new()),
            Arc::new(InMemoryScoreRepository::new()),
        )
    }

    fn with_auth_config_repos(
        auth_cfg: AuthConfig,
        custody_repo: Arc<dyn CustodyRepository>,
        orders: Arc<dyn OrderRepository>,
        score_repo: Arc<dyn ScoreRepository>,
    ) -> Self {
        let cfg = load_stellar_config();
        let http = Client::new();
        let sefaz = SefazValidator::new(http.clone(), None);
        let social = SocialValidator::new(http);
        let antifraude = Arc::new(AntiFraudeService::new(sefaz, social, score_repo));
        let auth = Arc::new(AuthService::with_antifraude(auth_cfg, antifraude.clone()));
        let custody = Arc::new(CustodyService::new(
            custody_repo,
            YieldCalculator::default(),
        ));
        let anchor = Arc::new(AnchorService::new(cfg));
        tracing::info!(
            fiat_rail = anchor.fiat_rail_name(),
            "anchor service (simulated rail = PIX EMV via Gatebox quando GATEBOX_BASE_URL está definido)",
        );
        Self {
            auth,
            antifraude,
            custody,
            anchor,
            orders,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::with_auth_config(AuthConfig::local_dev_open())
    }
}

fn load_stellar_config() -> StellarConfig {
    StellarConfig::from_env().unwrap_or_else(|_| StellarConfig {
        network: StellarNetwork::Testnet,
        anchor_url: std::env::var("APICASH_STELLAR_ANCHOR_URL")
            .unwrap_or_else(|_| "https://anchor.example.com".into()),
        asset_code: std::env::var("APICASH_STELLAR_ASSET_CODE").unwrap_or_else(|_| "BRLx".into()),
        horizon_url: std::env::var("APICASH_STELLAR_HORIZON_URL")
            .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".into()),
        secret_key: std::env::var("APICASH_STELLAR_SECRET_KEY")
            .unwrap_or_else(|_| "SANDBOX_DO_NOT_USE_REAL_SECRET".into()),
    })
}

fn env_enabled(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}
