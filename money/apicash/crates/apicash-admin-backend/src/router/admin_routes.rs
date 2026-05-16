//! Rotas sob `/admin/*`.

use axum::middleware;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;

use apicash_auth::middleware::verify_admin;

use crate::handlers::{
    dashboard_handler, dispute_admin_handler, orders_handler, report_handler, seller_handler,
    yield_handler,
};
use crate::state::AdminState;

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "apicash-admin-backend"
    }))
}

async fn ready() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ready",
        "service": "apicash-admin-backend"
    }))
}

pub fn admin_router(state: AdminState) -> Router {
    let auth = state.auth.clone();
    let under_admin = Router::new()
        .route("/admin/dashboard", get(dashboard_handler::get_dashboard))
        .route(
            "/admin/sellers/{id}/dashboard",
            get(seller_handler::get_seller_dashboard),
        )
        .route("/admin/orders", get(orders_handler::list_orders))
        .route("/admin/disputes", get(dispute_admin_handler::list_disputes))
        .route(
            "/admin/disputes/{id}",
            get(dispute_admin_handler::get_dispute),
        )
        .route(
            "/admin/disputes/{id}/resolve",
            post(dispute_admin_handler::resolve_dispute),
        )
        .route("/admin/reports/yield", get(yield_handler::get_yield_report))
        .route("/admin/users/score", get(report_handler::list_user_scores))
        .layer(middleware::from_fn(move |req, next| {
            let auth = auth.clone();
            async move { verify_admin(&auth, req, next).await }
        }));

    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .merge(under_admin)
        .with_state(state)
}
