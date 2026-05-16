//! Testes de rota administrativa (auth por API key).

use apicash_admin_backend::{admin_router, AdminState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn admin_auth_flow() {
    std::env::remove_var("APICASH_ADMIN_API_KEY");
    let app = admin_router(AdminState::new());
    let req = Request::builder()
        .uri("/admin/dashboard")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    std::env::set_var("APICASH_ADMIN_API_KEY", "integration-test-key");
    let req = Request::builder()
        .uri("/admin/dashboard")
        .header("x-api-key", "integration-test-key")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert!(v.get("total_volume_minor").is_some());
}
