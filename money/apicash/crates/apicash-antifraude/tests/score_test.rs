//! Integration-style tests for scoring and the anti-fraud service.

use std::sync::Arc;

use apicash_antifraude::{
    AntiFraudeService, BehavioralContext, DocumentStatus, DocumentType, InMemoryScoreRepository,
    LocalDocumentValidator, OnRampDecision, RiskLevel, ScoreCalculator, SocialAccountSnapshot,
    SocialValidator,
};
use reqwest::Client;
use rust_decimal::Decimal;
use uuid::Uuid;

fn make_service(repo: Arc<InMemoryScoreRepository>) -> AntiFraudeService {
    let doc_validator = Arc::new(LocalDocumentValidator::new());
    let social = SocialValidator::new(Client::new(), false);
    AntiFraudeService::new(doc_validator, social, repo)
}

// Seed enough history so first-tx penalty doesn't interfere in identity-focused tests.
async fn seed_neutral_history(repo: &InMemoryScoreRepository, uid: Uuid) {
    repo.seed_transactions(uid, 0, Decimal::ZERO, 5, Some(Decimal::from(200u32))).await;
}

#[tokio::test]
async fn cpf_regular_without_social_yields_review() {
    let uid = Uuid::new_v4();
    let repo = Arc::new(InMemoryScoreRepository::new());
    seed_neutral_history(&repo, uid).await;
    let svc = make_service(repo);

    let score = svc
        .calculate_score(uid, "52998224725", &[], None)
        .await
        .expect("score");

    // 350 (CPF) + 60 (clean history 5 tx) = 410 → High (250–499)
    assert_eq!(score.score, 410);
    assert_eq!(score.risk_level, RiskLevel::High);
    assert_eq!(score.decision, OnRampDecision::Review);
}

#[tokio::test]
async fn cpf_regular_and_old_social_yields_review() {
    let uid = Uuid::new_v4();
    let repo = Arc::new(InMemoryScoreRepository::new());
    seed_neutral_history(&repo, uid).await;
    let svc = make_service(repo);

    let links = vec!["https://instagram.com/old_user".to_string()];
    let score = svc
        .calculate_score(uid, "52998224725", &links, None)
        .await
        .expect("score");

    // 350 (CPF) + 180 (social) + 60 (history) = 590
    assert_eq!(score.score, 590);
    assert_eq!(score.risk_level, RiskLevel::Medium);
    assert_eq!(score.decision, OnRampDecision::Review);
}

#[tokio::test]
async fn irregular_cpf_blocks_even_with_social() {
    let uid = Uuid::new_v4();
    let repo = Arc::new(InMemoryScoreRepository::new());
    seed_neutral_history(&repo, uid).await;
    let svc = make_service(repo);

    let links = vec!["https://instagram.com/old_user".to_string()];
    let score = svc
        .calculate_score(uid, "00000000000", &links, None)
        .await
        .expect("score");

    // -320 (irregular) + 180 (social) + 60 (history) = -80 → clamped to 0
    assert_eq!(score.score, 0);
    assert_eq!(score.risk_level, RiskLevel::Critical);
    assert_eq!(score.decision, OnRampDecision::Block);
}

#[tokio::test]
async fn disputes_apply_strong_penalty() {
    let uid = Uuid::new_v4();
    let repo = Arc::new(InMemoryScoreRepository::new());
    repo.seed_disputes(uid, 3).await;
    seed_neutral_history(&repo, uid).await;

    let svc = make_service(repo);

    let links = vec!["https://instagram.com/old_user".to_string()];
    let score = svc
        .calculate_score(uid, "52998224725", &links, None)
        .await
        .expect("score");

    // 350 + 180 - 3*110 (disputes_by) - 150 (dispute_rate 3/5=60%>20%) = 50
    assert_eq!(score.score, 50);
    assert_eq!(score.decision, OnRampDecision::Block);
}

#[test]
fn calculator_direct_regular_and_social() {
    let uid = Uuid::nil();
    let social = vec![SocialAccountSnapshot {
        platform: "instagram".into(),
        handle: "x".into(),
        estimated_age_months: 12,
        name_consistent: true,
    }];
    let ctx = BehavioralContext {
        tx_count_total: 10,
        open_dispute_count: 0,
        disputes_as_counterparty: 0,
        dispute_rate: 0.0,
        tx_count_24h: 0,
        tx_volume_24h_brl: Decimal::ZERO,
        avg_tx_value: None,
        account_age_days: 30,
        current_tx_amount: None,
        company_age_months: None,
    };
    let s = ScoreCalculator::build_score(uid, DocumentType::Cpf, DocumentStatus::Valid, &social, &ctx);
    // 350 + 180 + 60 (clean history 5–29 tx) = 590
    assert_eq!(s.score, 590);
    assert_eq!(s.get_risk_recommendation(), "REVIEW");
}

#[test]
fn cnpj_ativa_established_company_scores_correctly() {
    let uid = Uuid::nil();
    let ctx = BehavioralContext {
        tx_count_total: 10,
        open_dispute_count: 0,
        disputes_as_counterparty: 0,
        dispute_rate: 0.0,
        tx_count_24h: 0,
        tx_volume_24h_brl: Decimal::ZERO,
        avg_tx_value: None,
        account_age_days: 30,
        current_tx_amount: None,
        company_age_months: Some(36), // 3 years old → established
    };
    let s = ScoreCalculator::build_score(uid, DocumentType::Cnpj, DocumentStatus::Valid, &[], &ctx);
    // 100 (CNPJ ATIVA) + 50 (>24 months) + 60 (clean history) = 210
    assert_eq!(s.score, 210);
    // 210 < THRESHOLD_HIGH_MIN (250) → Critical; CNPJ gets lower base trust than CPF by design
    assert_eq!(s.risk_level, RiskLevel::Critical);
}

#[test]
fn cnpj_ativa_new_company_gets_age_penalty() {
    let uid = Uuid::nil();
    let ctx = BehavioralContext {
        tx_count_total: 10,
        open_dispute_count: 0,
        disputes_as_counterparty: 0,
        dispute_rate: 0.0,
        tx_count_24h: 0,
        tx_volume_24h_brl: Decimal::ZERO,
        avg_tx_value: None,
        account_age_days: 30,
        current_tx_amount: None,
        company_age_months: Some(3), // 3 months old → new company penalty
    };
    let s = ScoreCalculator::build_score(uid, DocumentType::Cnpj, DocumentStatus::Valid, &[], &ctx);
    // 100 (CNPJ ATIVA) - 150 (<6 months) + 60 (clean history) = 10
    assert_eq!(s.score, 10);
    assert_eq!(s.decision, OnRampDecision::Block);
}

#[test]
fn cnpj_inativa_blocks() {
    let uid = Uuid::nil();
    let ctx = BehavioralContext {
        tx_count_total: 0,
        open_dispute_count: 0,
        disputes_as_counterparty: 0,
        dispute_rate: 0.0,
        tx_count_24h: 0,
        tx_volume_24h_brl: Decimal::ZERO,
        avg_tx_value: None,
        account_age_days: 0,
        current_tx_amount: None,
        company_age_months: Some(60),
    };
    let s = ScoreCalculator::build_score(uid, DocumentType::Cnpj, DocumentStatus::Invalid, &[], &ctx);
    // -200 (BAIXADA/SUSPENSA) + 50 (>24 months) - 40 (first tx) = clamped to 0
    assert_eq!(s.score, 0);
    assert_eq!(s.decision, OnRampDecision::Block);
}
