mod config;
mod handlers;
mod models;
mod notify;
mod router;
mod store;

use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .init();

    let cfg = config::Config::from_env();
    let state = router::build_state();
    let app = router::build_router(state);

    info!(addr = %cfg.bind_addr, "LogisticaHoldFy backend iniciando");
    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
