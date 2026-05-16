// Servidor Prometheus dedicado na porta 2112 (como no Go)
use axum::{routing::get, Router};
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

async fn prometheus_metrics() -> (axum::http::StatusCode, String) {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    if encoder.encode(&metric_families, &mut buffer).is_err() {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, String::new());
    }
    (
        axum::http::StatusCode::OK,
        String::from_utf8(buffer).unwrap_or_default(),
    )
}

/// Inicia servidor de métricas Prometheus em `port` (default 2112).
pub async fn start_metrics_server(port: &str) -> anyhow::Result<Option<tokio::task::JoinHandle<()>>> {
    let metrics_router = Router::new().route("/metrics", get(prometheus_metrics));

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("Failed to bind metrics server on {}: {}", addr, e);
            return Ok(None);
        }
    };

    info!("Metrics server started on http://localhost:{}/metrics", port);

    let handle = tokio::spawn(async move {
        let axum = axum::serve(listener, metrics_router);
        if let Err(e) = axum.await {
            tracing::warn!("Metrics server error: {}", e);
        }
    });

    Ok(Some(handle))
}
