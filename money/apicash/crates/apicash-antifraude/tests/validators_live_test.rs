//! Tests for the document and social validators.
//!
//! HTTP tests use a local mockito server instead of real providers.
//! Local tests exercise the pure Rust mathematical digit-check algorithm.

use apicash_antifraude::{
    DocumentStatus, DocumentType, DocumentValidator, HttpDocumentValidator,
    LocalDocumentValidator, SocialValidator,
};
use reqwest::Client;

// ─── HTTP document validator ──────────────────────────────────────────────────

#[tokio::test]
async fn http_regular_cpf_from_provider() {
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/v1/cpf/52998224725")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"situacao":"Regular","nome":"Fulano de Tal"}"#)
        .create_async()
        .await;

    let validator = HttpDocumentValidator::new(Client::new(), server.url(), None);
    let status = validator.validate("529.982.247-25", DocumentType::Cpf).await.unwrap();
    assert_eq!(status, DocumentStatus::Valid);
}

#[tokio::test]
async fn http_irregular_cpf_from_provider() {
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/v1/cpf/12345678900")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"situacao":"Suspensa"}"#)
        .create_async()
        .await;

    let validator = HttpDocumentValidator::new(Client::new(), server.url(), None);
    let status = validator.validate("12345678900", DocumentType::Cpf).await.unwrap();
    assert_eq!(status, DocumentStatus::Invalid);
}

#[tokio::test]
async fn http_provider_error_yields_unknown() {
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/v1/cpf/11122233344")
        .with_status(503)
        .create_async()
        .await;

    let validator = HttpDocumentValidator::new(Client::new(), server.url(), None);
    let status = validator.validate("11122233344", DocumentType::Cpf).await.unwrap();
    assert_eq!(status, DocumentStatus::Unknown);
}

#[tokio::test]
async fn http_bearer_token_is_sent() {
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/v1/cpf/52998224725")
        .match_header("authorization", "Bearer test-token-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"situacao":"Regular"}"#)
        .create_async()
        .await;

    let validator = HttpDocumentValidator::new(
        Client::new(),
        server.url(),
        Some("test-token-123".into()),
    );
    let status = validator.validate("52998224725", DocumentType::Cpf).await.unwrap();
    assert_eq!(status, DocumentStatus::Valid);
}

// ─── Local (mathematical digit check) ────────────────────────────────────────

#[tokio::test]
async fn local_valid_cpf_yields_valid() {
    let validator = LocalDocumentValidator::new();
    let status = validator.validate("529.982.247-25", DocumentType::Cpf).await.unwrap();
    assert_eq!(status, DocumentStatus::Valid);
}

#[tokio::test]
async fn local_invalid_cpf_all_zeros_yields_invalid() {
    let validator = LocalDocumentValidator::new();
    let status = validator.validate("00000000000", DocumentType::Cpf).await.unwrap();
    assert_eq!(status, DocumentStatus::Invalid);
}

#[tokio::test]
async fn local_wrong_check_digits_yields_invalid() {
    let validator = LocalDocumentValidator::new();
    // Last two digits flipped → fails digit check
    let status = validator.validate("52998224752", DocumentType::Cpf).await.unwrap();
    assert_eq!(status, DocumentStatus::Invalid);
}

#[tokio::test]
async fn local_valid_cnpj_yields_valid() {
    let validator = LocalDocumentValidator::new();
    // 11.222.333/0001-81 — mathematically valid
    let status = validator.validate("11222333000181", DocumentType::Cnpj).await.unwrap();
    assert_eq!(status, DocumentStatus::Valid);
}

#[tokio::test]
async fn local_invalid_cnpj_yields_invalid() {
    let validator = LocalDocumentValidator::new();
    let status = validator.validate("00000000000000", DocumentType::Cnpj).await.unwrap();
    assert_eq!(status, DocumentStatus::Invalid);
}

// ─── Social ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn social_reachable_profile_yields_snapshot() {
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("HEAD", "/johndoe")
        .with_status(200)
        .create_async()
        .await;

    let profile_url = format!("{}/johndoe", server.url());
    let validator = SocialValidator::new(Client::new(), true);
    let results = validator.validate_links(&[profile_url]).await.unwrap();

    let r = &results[0];
    assert!(r.snapshot.is_some(), "reachable profile should yield a snapshot");
    assert_eq!(r.snapshot.as_ref().unwrap().name_consistent, true);
}

#[tokio::test]
async fn social_404_yields_no_snapshot() {
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("HEAD", "/ghost-user")
        .with_status(404)
        .create_async()
        .await;

    let profile_url = format!("{}/ghost-user", server.url());
    let validator = SocialValidator::new(Client::new(), true);
    let results = validator.validate_links(&[profile_url]).await.unwrap();

    let r = &results[0];
    assert!(r.snapshot.is_none(), "404 profile should have no snapshot");
    assert!(r.error.is_some());
}

#[tokio::test]
async fn social_check_disabled_uses_heuristic() {
    // With check_enabled=false, URL containing "old" → estimated_age_months=12
    let validator = SocialValidator::new(Client::new(), false);
    let url = "https://instagram.com/old_user".to_string();
    let results = validator.validate_links(&[url]).await.unwrap();

    let snap = results[0].snapshot.as_ref().unwrap();
    assert_eq!(snap.estimated_age_months, 12);
}
