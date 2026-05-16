//! Integration-style tests against the composed router.

use std::sync::Arc;

use apicash_core::{create_router, AppState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let app = create_router(Arc::new(AppState::default()));
    let res = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn orders_not_found() {
    let app = create_router(Arc::new(AppState::default()));
    let res = app
        .oneshot(
            Request::builder()
                .uri("/orders/00000000-0000-0000-0000-000000000001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
