//! Binário do backend administrativo — escuta **3001** (separado da API pública `apicash-core`).

use apicash_admin_backend::{admin_router, AdminState};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    apicash_shared::logging::init_tracing("apicash_admin_backend=info,tower_http=info");

    let state = AdminState::connect_from_env().await?;
    let app = admin_router(state).layer(TraceLayer::new_for_http());

    let addr: std::net::SocketAddr = "0.0.0.0:3001".parse().expect("parse bind addr");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind admin listener");

    tracing::info!(%addr, "apicash-admin-backend listening");

    axum::serve(listener, app).await?;
    Ok(())
}
