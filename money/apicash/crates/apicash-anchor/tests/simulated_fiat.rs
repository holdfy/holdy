//! Fiat rail `simulated`: PIX EMV vem **só** do Gatebox (sem placeholder local).

use std::sync::Mutex;

use apicash_anchor::{AnchorService, StellarConfig, StellarNetwork};
use apicash_shared::Money;
use rust_decimal::Decimal;
use wiremock::matchers::{method, path};
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate};

static SERIAL: Mutex<()> = Mutex::new(());

fn clear_gatebox_env() {
    for k in [
        "APICASH_GATEBOX_ENABLED",
        "GATEBOX_BASE_URL",
        "GATEBOX_API_KEY",
    ] {
        unsafe { std::env::remove_var(k) };
    }
}

#[tokio::test]
async fn simulated_deposit_requires_gatebox() {
    let _guard = SERIAL.lock().expect("mutex");
    clear_gatebox_env();

    let prev_rail = std::env::var("APICASH_FIAT_RAIL").ok();
    std::env::set_var("APICASH_FIAT_RAIL", "simulated");

    let cfg = StellarConfig {
        network: StellarNetwork::Testnet,
        anchor_url: "https://ignored.example.invalid".into(),
        asset_code: "BRLx".into(),
        horizon_url: "https://ignored.example.invalid".into(),
        secret_key: "sandbox".into(),
    };
    let svc = AnchorService::new(cfg);
    let err = svc
        .deposit_pix(
            Money::new(Decimal::new(1575, 2)),
            "order:test-sim-deposit".into(),
        )
        .await
        .expect_err("expected config error without Gatebox");
    let msg = err.to_string();
    assert!(
        msg.contains("GATEBOX") || msg.contains("Gatebox"),
        "unexpected error: {msg}"
    );

    match prev_rail {
        Some(v) => std::env::set_var("APICASH_FIAT_RAIL", v),
        None => unsafe { std::env::remove_var("APICASH_FIAT_RAIL") },
    }
}

#[tokio::test]
async fn simulated_deposit_returns_gatebox_emv() {
    let _guard = SERIAL.lock().expect("mutex");
    clear_gatebox_env();

    let server = MockServer::start().await;
    let emv = "00020126580014br.gov.bcb.pix0136testemv0000000000";
    Mock::given(method("POST"))
        .and(path("/api/v1/pix/qrcode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": 200,
            "qrCode": emv,
            "txId": "tx-mock-1",
            "transactionId": "tx-mock-1",
            "gateway": "sulcred"
        })))
        .mount(&server)
        .await;

    let prev_rail = std::env::var("APICASH_FIAT_RAIL").ok();
    std::env::set_var("APICASH_FIAT_RAIL", "simulated");
    std::env::set_var("GATEBOX_BASE_URL", server.uri());

    let cfg = StellarConfig {
        network: StellarNetwork::Testnet,
        anchor_url: "https://ignored.example.invalid".into(),
        asset_code: "BRLx".into(),
        horizon_url: "https://ignored.example.invalid".into(),
        secret_key: "sandbox".into(),
    };
    let svc = AnchorService::new(cfg);

    let r = svc
        .deposit_pix(
            Money::new(Decimal::new(1575, 2)),
            "order:test-sim-deposit".into(),
        )
        .await
        .expect("simulated deposit with Gatebox mock");

    assert_eq!(r.fiat_rail, "simulated");
    assert_eq!(r.pix_br_code.as_deref(), Some(emv));
    assert!(r.stellar_tx_hash.starts_with("mock_stellar_sim_"));
    assert_eq!(r.status, "pending");

    match prev_rail {
        Some(v) => std::env::set_var("APICASH_FIAT_RAIL", v),
        None => unsafe { std::env::remove_var("APICASH_FIAT_RAIL") },
    }
    unsafe { std::env::remove_var("GATEBOX_BASE_URL") };
}
