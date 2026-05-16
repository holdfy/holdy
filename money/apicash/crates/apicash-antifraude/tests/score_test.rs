//! Integration-style tests for scoring and the anti-fraud service (uses `mock` validators).

use std::sync::Arc;

use apicash_antifraude::{
    AntiFraudeService, InMemoryScoreRepository, OnRampDecision, RiskLevel, ScoreCalculator,
    SefazPersonStatus, SefazValidator, SocialValidator,
};
use reqwest::Client;
use uuid::Uuid;

#[tokio::test]
async fn cpf_regular_without_social_yields_review() {
    let uid = Uuid::new_v4();
    let sefaz = SefazValidator::new(Client::new(), None);
    let social = SocialValidator::new(Client::new());
    let repo = Arc::new(InMemoryScoreRepository::new());
    let svc = AntiFraudeService::new(sefaz, social, repo);

    let score = svc
        .calculate_score(uid, "52998224725", &[])
        .await
        .expect("score");

    assert_eq!(score.score, 350);
    assert_eq!(score.risk_level, RiskLevel::High);
    assert_eq!(score.decision, OnRampDecision::Review);
}

#[tokio::test]
async fn cpf_regular_and_old_social_yields_review() {
    let uid = Uuid::new_v4();
    let sefaz = SefazValidator::new(Client::new(), None);
    let social = SocialValidator::new(Client::new());

    let repo = Arc::new(InMemoryScoreRepository::new());
    let svc = AntiFraudeService::new(sefaz, social, repo.clone());

    let links = vec!["https://instagram.com/old_user".to_string()];
    let score = svc
        .calculate_score(uid, "52998224725", &links)
        .await
        .expect("score");

    assert_eq!(score.score, 530); // 350 + 180
    assert_eq!(score.risk_level, RiskLevel::Medium);
    assert_eq!(score.decision, OnRampDecision::Review);
}

#[tokio::test]
async fn irregular_cpf_blocks_even_with_social() {
    let uid = Uuid::new_v4();
    let sefaz = SefazValidator::new(Client::new(), None);
    let social = SocialValidator::new(Client::new());
    let repo = Arc::new(InMemoryScoreRepository::new());
    let svc = AntiFraudeService::new(sefaz, social, repo);

    let links = vec!["https://instagram.com/old_user".to_string()];
    let score = svc
        .calculate_score(uid, "00000000000", &links)
        .await
        .expect("score");

    // 150 (social) - 280 (irregular CPF) -> negative clamped to 0
    assert_eq!(score.score, 0);
    assert_eq!(score.risk_level, RiskLevel::Critical);
    assert_eq!(score.decision, OnRampDecision::Block);
}

#[tokio::test]
async fn disputes_apply_strong_penalty() {
    let uid = Uuid::new_v4();
    let repo = Arc::new(InMemoryScoreRepository::new());
    repo.seed_disputes(uid, 3).await;

    let sefaz = SefazValidator::new(Client::new(), None);
    let social = SocialValidator::new(Client::new());
    let svc = AntiFraudeService::new(sefaz, social, repo);

    let links = vec!["https://instagram.com/old_user".to_string()];
    let score = svc
        .calculate_score(uid, "52998224725", &links)
        .await
        .expect("score");

    // 350 + 180 - 3*110 = 200
    assert_eq!(score.score, 200);
    assert_eq!(score.decision, OnRampDecision::Block);
}

#[test]
fn calculator_direct_regular_and_social() {
    let uid = Uuid::nil();
    let social = vec![apicash_antifraude::SocialAccountSnapshot {
        platform: "instagram".into(),
        handle: "x".into(),
        estimated_age_months: 12,
        name_consistent: true,
    }];
    let s = ScoreCalculator::build_score(uid, SefazPersonStatus::Regular, &social, 0);
    assert_eq!(s.score, 530);
    assert_eq!(s.get_risk_recommendation(), "REVIEW");
}
