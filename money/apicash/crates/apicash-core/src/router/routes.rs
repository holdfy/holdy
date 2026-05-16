//! Composed Axum router: tracing, optional auth, REST routes.

use std::sync::Arc;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_http::trace::TraceLayer;

use crate::handlers::{auth_handler, custody_handler, order_handler, payment_handler};
use crate::middleware::auth_middleware;
use crate::state::AppState;

/// Builds the HTTP application with shared [`AppState`].
pub fn create_router(state: Arc<AppState>) -> Router {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(12)
            .finish()
            .expect("governor config"),
    );

    let auth_routes = Router::new()
        .route("/login", post(auth_handler::login))
        .route("/refresh", post(auth_handler::refresh))
        .layer(GovernorLayer::new(governor_conf.clone()))
        .with_state(state.clone());

    let protected = Router::new()
        .route("/orders", post(order_handler::create_order))
        .route("/orders/{id}", get(order_handler::get_order))
        .route(
            "/orders/{id}/settle",
            post(order_handler::settle_order_manual),
        )
        .route("/orders/{id}/off-ramp", post(order_handler::order_off_ramp))
        .route("/risk/score", post(order_handler::calculate_risk_score))
        .route("/payments/pix", post(payment_handler::create_pix_payment))
        .route("/custody/release", post(custody_handler::release_custody))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Internal service-to-service routes.
    //
    // Security decision: keep the public `POST /risk/score` bound to end-user JWT (via middleware),
    // but allow the WhatsApp Agent to call a dedicated internal endpoint using a service API key
    // (`X-API-Key` = `APICASH_API_KEY`). This avoids requiring a full end-user auth dance before
    // the bot can pre-calculate score, while keeping the public surface locked down.
    let internal = Router::new()
        .route(
            "/internal/risk/score",
            post(order_handler::calculate_risk_score_internal),
        )
        .route(
            "/internal/orders/settle",
            post(order_handler::settle_order_internal),
        )
        .layer(GovernorLayer::new(governor_conf.clone()))
        .with_state(state.clone());

    Router::new()
        .route("/", get(order_handler::root))
        .route("/health", get(order_handler::health))
        .route("/ready", get(order_handler::ready))
        .nest("/auth", auth_routes)
        .merge(internal)
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
