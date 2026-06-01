use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handlers::{
    add_event, add_preset_step, create_tracker, get_tracker, health, list_presets,
    list_trackers, track_shipment, AppState,
};
use crate::notify::WhatsAppNotifier;
use crate::store::TrackerStore;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/presets", get(list_presets))
        .route("/trackers", get(list_trackers).post(create_tracker))
        .route("/trackers/{code}", get(get_tracker))
        .route("/trackers/{code}/events", post(add_event))
        .route("/trackers/{code}/presets", post(add_preset_step))
        // Alias compatível com apicash-core GET /logistics/tracking/{code}
        .route("/logistics/tracking/{code}", get(track_shipment))
        .route("/tracking/{code}", get(track_shipment))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub fn build_state() -> AppState {
    AppState {
        store: Arc::new(TrackerStore::new()),
        notifier: Arc::new(WhatsAppNotifier::from_env()),
    }
}
