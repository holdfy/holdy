//! HTTP-backed Anchor tests against a local mock server (no production anchors).

use apicash_anchor::client::AnchorClient;
use apicash_shared::Money;
use reqwest::Client;
use rust_decimal::Decimal;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn deposit_maps_anchor_json_to_on_ramp() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/pix/deposit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "transaction_id": "tx-provi",
            "stellar_tx_hash": "abc123stellar",
            "status": "pending_settlement",
            "pix_br_code": "00020126360014BR.GOV.BCB.PIX0114test",
        })))
        .mount(&server)
        .await;

    let http = Client::new();
    let client = AnchorClient::new(http, server.uri(), "BRLx".into());
    let r = client
        .request_deposit_pix(Money::new(Decimal::new(5000, 2)), "order:uuid")
        .await
        .expect("deposit");

    assert_eq!(r.fiat_rail, "anchor");
    assert_eq!(r.transaction_id.as_deref(), Some("tx-provi"));
    assert_eq!(r.stellar_tx_hash, "abc123stellar");
    assert_eq!(
        r.pix_br_code.as_deref(),
        Some("00020126360014BR.GOV.BCB.PIX0114test")
    );
}

#[tokio::test]
async fn withdraw_maps_anchor_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/pix/withdraw"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "transaction_id": "w1",
            "tx_hash": "withdraw-hash",
            "status": "pix_queued",
            "received_pix": "25.00"
        })))
        .mount(&server)
        .await;

    let http = Client::new();
    let client = AnchorClient::new(http, server.uri(), "BRLx".into());
    let r = client
        .request_withdraw_pix(Money::new(Decimal::new(2500, 2)), "pix@email.com")
        .await
        .expect("withdraw");

    assert_eq!(r.tx_hash, "withdraw-hash");
    assert_eq!(r.status, "pix_queued");
}
