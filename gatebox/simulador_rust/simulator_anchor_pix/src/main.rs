//! Simulador do Etherfuse Anchor para ambiente testnet.
//!
//! Expõe a mesma API REST que o Etherfuse real terá:
//!   POST /v1/pix/deposit          — on-ramp: cria intenção de depósito PIX
//!   GET  /v1/pix/transaction/:id  — polling de settlement (auto-completa após delay)
//!   POST /v1/pix/withdraw         — off-ramp: simula envio de PIX ao vendedor
//!   GET  /health
//!
//! No APICash: APICASH_FIAT_RAIL=anchor + APICASH_STELLAR_ANCHOR_URL=http://<host>:<port>
//! Isso faz o AnchorRail (código de produção) falar com este simulador — zero alteração no APICash.
//!
//! Env vars:
//!   PORT                       (padrão: 8093)
//!   ANCHOR_SIM_AUTO_SETTLE_MS  (padrão: 3000) — ms até settlement completar
//!   ANCHOR_SIM_ASSET_CODE      (padrão: BRLx)
//!   GATEBOX_BASE_URL            — se presente, obtém QR EMV real do Gatebox/Sulcred
//!   GATEBOX_API_KEY             — bearer auth para o Gatebox
//!   GATEBOX_CUSTOMER_NAME       — pagador no QR (padrão: "HoldFy Testnet")
//!   GATEBOX_CUSTOMER_DOCUMENT   — CPF/CNPJ do pagador

mod gatebox;
mod handlers;
mod models;
mod state;

use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tracing::info;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,simulator_anchor_pix=debug".into()),
        )
        .init();

    let state = AppState::from_env();
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8093);

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/v1/pix/deposit", post(handlers::deposit))
        .route("/v1/pix/transaction/:id", get(handlers::transaction_status))
        .route("/v1/pix/withdraw", post(handlers::withdraw))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(port, "simulator-anchor-pix iniciando");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
