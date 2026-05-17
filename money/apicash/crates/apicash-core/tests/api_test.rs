//! Integration-style tests against the composed router.

use std::sync::Arc;

use serial_test::serial;

use apicash_core::{create_router, AppState};
use apicash_auth::AuthConfig;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

fn clear_x402_env() {
    std::env::remove_var("APICASH_X402_REQUIRED");
    std::env::remove_var("X402_FACILITATOR_URL");
    std::env::remove_var("X402_PAY_TO");
    std::env::remove_var("X402_PRICE_USDC");
    std::env::remove_var("APICASH_PUBLIC_BASE_URL");
}

fn test_auth_config() -> AuthConfig {
    AuthConfig {
        jwt_secret: "test_jwt_secret_for_x402____________".into(),
        jwt_issuer: "apicash".into(),
        jwt_ttl_secs: 3600,
        jwt_refresh_ttl_secs: 604_800,
        auth_disabled: true,
    }
}

fn test_state() -> Arc<AppState> {
    std::env::remove_var("APICASH_REQUIRE_TESTNET");
    clear_x402_env();
    Arc::new(AppState::with_auth_config(test_auth_config()))
}

fn test_state_without_clearing_x402() -> Arc<AppState> {
    std::env::remove_var("APICASH_REQUIRE_TESTNET");
    Arc::new(AppState::with_auth_config(test_auth_config()))
}

#[tokio::test]
#[serial]
async fn health_returns_ok() {
    let app = create_router(test_state());
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
#[serial]
async fn orders_not_found_without_x402() {
    let app = create_router(test_state());
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

#[tokio::test]
#[serial]
async fn orders_return_402_when_x402_required_and_no_payment() {
    std::env::remove_var("APICASH_REQUIRE_TESTNET");
    std::env::set_var("APICASH_X402_REQUIRED", "1");
    std::env::set_var("X402_FACILITATOR_URL", "https://facilitator.x402.rs");
    std::env::set_var(
        "X402_PAY_TO",
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    );
    std::env::set_var("X402_PRICE_USDC", "0.01");
    std::env::set_var("APICASH_PUBLIC_BASE_URL", "http://127.0.0.1:3000");

    let app = create_router(test_state_without_clearing_x402());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/orders/00000000-0000-0000-0000-000000000001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    clear_x402_env();

    assert_eq!(res.status(), StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
#[serial]
async fn internal_risk_score_skips_x402() {
    std::env::set_var("APICASH_X402_REQUIRED", "1");
    std::env::set_var("X402_FACILITATOR_URL", "https://facilitator.x402.rs");
    std::env::set_var(
        "X402_PAY_TO",
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    );
    std::env::set_var("APICASH_API_KEY", "test-service-key");

    let app = create_router(test_state_without_clearing_x402());
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/risk/score")
                .header("x-api-key", "test-service-key")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"user_id":"00000000-0000-0000-0000-000000000001","cpf":"12345678909"}"#,
                ))
                .unwrap(),
        )
        .await
        .expect("response");

    clear_x402_env();
    std::env::remove_var("APICASH_API_KEY");

    assert_ne!(res.status(), StatusCode::PAYMENT_REQUIRED);
}
