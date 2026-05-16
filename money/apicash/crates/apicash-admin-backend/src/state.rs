//! Estado compartilhado: serviços de domínio + repositórios do painel admin.

use std::sync::Arc;

use apicash_antifraude::{
    AntiFraudeService, InMemoryScoreRepository, PostgresScoreRepository, SefazValidator,
    SocialValidator,
};
use apicash_auth::{AuthConfig, AuthService};
use apicash_core::state::{InMemoryOrderRepository, OrderRepository, PostgresOrderRepository};
use apicash_custody::{
    CustodyRepository, CustodyService, InMemoryCustodyRepository, PostgresCustodyRepository,
    YieldCalculator,
};
use apicash_disputes::repository::{
    DisputeRepository, InMemoryDisputeRepository, PostgresDisputeRepository,
};
use apicash_disputes::service::{DisputeService, NoopDisputeEventSink};
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;

/// Estado injetado nos handlers Axum.
#[derive(Clone)]
pub struct AdminState {
    pub auth: Arc<AuthService>,
    pub custody: Arc<CustodyService>,
    pub antifraude: Arc<AntiFraudeService>,
    pub disputes: Arc<DisputeService>,
    pub orders: Arc<dyn OrderRepository>,
}

impl AdminState {
    /// Monta serviços in-memory (dev); produção pode compartilhar pools SQLx com `apicash-core`.
    pub fn new() -> Self {
        let http = Client::new();
        let sefaz = SefazValidator::new(http.clone(), None);
        let social = SocialValidator::new(http);
        let score_repo = Arc::new(InMemoryScoreRepository::new());
        let antifraude = Arc::new(AntiFraudeService::new(sefaz, social, score_repo));

        let auth_cfg = AuthConfig::from_env();
        let auth = Arc::new(AuthService::with_antifraude(auth_cfg, antifraude.clone()));

        let custody_repo = Arc::new(InMemoryCustodyRepository::new());
        let custody = Arc::new(CustodyService::new(
            custody_repo,
            YieldCalculator::default(),
        ));

        let dispute_repo: Arc<dyn DisputeRepository> = Arc::new(InMemoryDisputeRepository::new());
        let disputes = Arc::new(DisputeService::new(
            dispute_repo,
            custody.clone(),
            Arc::new(NoopDisputeEventSink),
            Default::default(),
        ));

        Self {
            auth,
            custody,
            antifraude,
            disputes,
            orders: Arc::new(InMemoryOrderRepository::new()),
        }
    }

    pub async fn connect_from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let want_pg = env_enabled("APICASH_ADMIN_PG")
            || env_enabled("APICASH_ORDERS_PG")
            || env_enabled("APICASH_CUSTODY_PG");

        if !want_pg {
            tracing::info!("admin: in-memory (set APICASH_ADMIN_PG=1 for Postgres)");
            return Ok(Self::new());
        }

        let Some(url) = std::env::var("DATABASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
        else {
            return Err("APICASH_ADMIN_PG=1 requires DATABASE_URL (postgresql://...)".into());
        };

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(url.trim())
            .await?;

        let http = Client::new();
        let sefaz = SefazValidator::new(http.clone(), None);
        let social = SocialValidator::new(http);
        let score_repo = Arc::new(PostgresScoreRepository::new(pool.clone()));
        let antifraude = Arc::new(AntiFraudeService::new(sefaz, social, score_repo));

        let auth_cfg = AuthConfig::from_env();
        let auth = Arc::new(AuthService::with_antifraude(auth_cfg, antifraude.clone()));

        let custody_repo: Arc<dyn CustodyRepository> =
            Arc::new(PostgresCustodyRepository::new(pool.clone()));
        let custody = Arc::new(CustodyService::new(
            custody_repo,
            YieldCalculator::default(),
        ));

        let dispute_repo: Arc<dyn DisputeRepository> =
            Arc::new(PostgresDisputeRepository::new(pool.clone()));
        let disputes = Arc::new(DisputeService::new(
            dispute_repo,
            custody.clone(),
            Arc::new(NoopDisputeEventSink),
            Default::default(),
        ));

        let orders: Arc<dyn OrderRepository> = Arc::new(PostgresOrderRepository::new(pool));

        tracing::info!("admin: Postgres repositories enabled");
        Ok(Self {
            auth,
            custody,
            antifraude,
            disputes,
            orders,
        })
    }
}

impl Default for AdminState {
    fn default() -> Self {
        Self::new()
    }
}

fn env_enabled(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}
