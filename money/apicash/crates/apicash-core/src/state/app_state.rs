//! Shared application state: domain services + order persistence.

use std::sync::Arc;

use apicash_anchor::{AnchorService, StellarConfig, StellarNetwork};
use apicash_antifraude::{
    AntiFraudeService, CachedDocumentValidator, DocumentCache, DocumentValidator,
    HttpDocumentValidator, InMemoryDocumentCache, InMemoryScoreRepository, LocalDocumentValidator,
    PostgresDocumentCache, PostgresScoreRepository, ReputationService, ScoreRepository,
    SocialValidator,
};
use apicash_auth::{AuthConfig, AuthService};
use apicash_custody::{
    CustodyRepository, CustodyService, InMemoryCustodyRepository, PostgresCustodyRepository,
    YieldCalculator,
};
use apicash_importer::ImporterService;
use apicash_logistics::LogisticsService;
use apicash_shared::Order;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

use super::order_repository::{InMemoryOrderRepository, OrderRepository, PostgresOrderRepository};
use super::proposal_repository::{InMemoryProposalRepository, ProposalRepository};

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
    pub reputation: Arc<ReputationService>,
    pub custody: Arc<CustodyService>,
    pub anchor: Arc<AnchorService>,
    pub orders: Arc<dyn OrderRepository>,
    pub proposals: Arc<dyn ProposalRepository>,
    pub importer: Arc<ImporterService>,
    pub logistics: Arc<LogisticsService>,
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
        let state = Self::with_auth_config_repos(
            AuthConfig::from_env(),
            custody_repo,
            orders,
            score_repo,
            pool,
        );
        Ok(state)
    }

    /// Permite testes com [`AuthConfig::local_dev_open`] sem depender de env.
    pub fn with_auth_config(auth_cfg: AuthConfig) -> Self {
        Self::with_auth_config_repos(
            auth_cfg,
            Arc::new(InMemoryCustodyRepository::new()),
            Arc::new(InMemoryOrderRepository::new()),
            Arc::new(InMemoryScoreRepository::new()),
            None,
        )
    }

    fn with_auth_config_repos(
        auth_cfg: AuthConfig,
        custody_repo: Arc<dyn CustodyRepository>,
        orders: Arc<dyn OrderRepository>,
        score_repo: Arc<dyn ScoreRepository>,
        pool: Option<sqlx::PgPool>,
    ) -> Self {
        let cfg = load_stellar_config();
        let http = Client::new();
        let sefaz_url = std::env::var("SEFAZ_API_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let sefaz_token = std::env::var("SEFAZ_API_TOKEN")
            .or_else(|_| std::env::var("SEFAZ_CLIENT_SECRET"))
            .ok()
            .filter(|s| !s.trim().is_empty());
        let social_check = std::env::var("SOCIAL_CHECK_ENABLED")
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

        // Build interchangeable document validator with cache (TTL 24h).
        // Use HTTP provider when SEFAZ_API_URL is configured; otherwise local math.
        let inner_validator: Arc<dyn DocumentValidator> = if let Some(url) = sefaz_url {
            tracing::info!("document_validator: HTTP provider at {url}");
            Arc::new(HttpDocumentValidator::new(http.clone(), url, sefaz_token))
        } else {
            tracing::info!("document_validator: local mathematical validation (CPF/CNPJ)");
            Arc::new(LocalDocumentValidator::new())
        };
        let want_doc_cache_pg = env_enabled("APICASH_DOC_CACHE_PG");
        let doc_cache: Arc<dyn DocumentCache> = match (want_doc_cache_pg, pool.clone()) {
            (true, Some(p)) => {
                tracing::info!("document_cache: Postgres (APICASH_DOC_CACHE_PG + DATABASE_URL)");
                Arc::new(PostgresDocumentCache::new(p)) as Arc<dyn DocumentCache>
            }
            _ => {
                tracing::info!(
                    "document_cache: in-memory (set APICASH_DOC_CACHE_PG=1 + DATABASE_URL for Postgres)"
                );
                Arc::new(InMemoryDocumentCache::new()) as Arc<dyn DocumentCache>
            }
        };
        let doc_validator = Arc::new(CachedDocumentValidator::new(
            inner_validator,
            doc_cache,
            std::time::Duration::from_secs(86_400), // 24h TTL
        ));

        let social = SocialValidator::new(http, social_check);
        let antifraude = Arc::new(AntiFraudeService::new(doc_validator, social, score_repo.clone()));
        let reputation = Arc::new(ReputationService::new(score_repo));
        let auth = Arc::new(AuthService::with_antifraude(auth_cfg, antifraude.clone()));
        let custody = Arc::new(CustodyService::new(
            custody_repo,
            YieldCalculator::default(),
        ));
        let anchor = Arc::new(AnchorService::new(cfg));
        let proposals = Arc::new(InMemoryProposalRepository::new());
        let importer = Arc::new(ImporterService::new());
        let logistics = Arc::new(build_logistics_service());
        tracing::info!(
            fiat_rail = anchor.fiat_rail_name(),
            "anchor service (simulated rail = PIX EMV via Gatebox quando GATEBOX_BASE_URL está definido)",
        );
        Self {
            auth,
            antifraude,
            reputation,
            custody,
            anchor,
            orders,
            proposals,
            importer,
            logistics,
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

fn build_logistics_service() -> LogisticsService {
    match LogisticsService::from_env() {
        Ok(svc) => {
            tracing::info!("logistics: Melhor Envio configurado (MELHOR_ENVIO_TOKEN presente)");
            svc
        }
        Err(_) => {
            tracing::warn!(
                "logistics: MELHOR_ENVIO_TOKEN ausente — cotações retornarão erro 500 em produção"
            );
            // Build a no-op service that fails gracefully at request time.
            let token = "MISSING_TOKEN".to_string();
            LogisticsService::new(apicash_logistics::MelhorEnvioClient::new(token, true))
        }
    }
}
