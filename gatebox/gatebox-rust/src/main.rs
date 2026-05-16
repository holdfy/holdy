mod config;
mod metrics_server;
mod observabilidade;
mod openapi;
mod server;
mod shared;

use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    config::init();

    observabilidade::init_observability(
        env::var("SERVICE_NAME").unwrap_or_else(|_| "gatebox-rust".to_string()),
        env::var("SERVICE_VERSION").unwrap_or_else(|_| "1.0.0".to_string()),
    )?;

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let mut app = server::App::new();
    if let Err(e) = app.start().await {
        info!("DB not configured or unavailable (run with POSTGRESQL_WRITE_URL/POSTGRESQL_READ_URL for full API): {}", e);
        info!("Serving health-only on port {}", port);

        let metrics_port = env::var("METRICS_PORT").unwrap_or_else(|_| "2112".to_string());
        let enable_metrics = env::var("ENABLE_METRICS").unwrap_or_else(|_| "true".to_string());
        if enable_metrics.to_lowercase() != "false" {
            if let Ok(Some(_h)) = metrics_server::start_metrics_server(&metrics_port).await {
                info!(
                    "Métricas (modo apenas health): http://localhost:{}/metrics",
                    metrics_port
                );
            }
        }

        let fallback = axum::Router::new().route("/health", axum::routing::get(|| async {
            axum::Json(serde_json::json!({
                "status": "healthy",
                "service": "gatebox-rust",
                "version": "1.0.0"
            }))
        }));
        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, fallback).await?;
        return Ok(());
    }

    app.run(&port).await?;
    app.stop().await;
    Ok(())
}
