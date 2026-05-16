// Converted from gateboxgo/utils/observabilidade/middleware.go (HTTP metrics for Axum)
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tower_http::timeout::TimeoutLayer;

use crate::observabilidade::metrics;

#[allow(dead_code)]
pub fn timeout_layer(timeout: std::time::Duration) -> TimeoutLayer {
    TimeoutLayer::new(timeout)
}

/// Middleware that records HTTP metrics. Use with tower::ServiceBuilder and a captured usecase.
pub async fn enhanced_http_metrics_layer(
    usecase: String,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(request).await;
    let status_code = response.status().as_u16();
    let duration = start.elapsed();

    metrics::record_http_request(&usecase, &method, &path, duration, status_code);

    response
}
