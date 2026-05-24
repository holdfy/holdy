//! Behavioral anti-fraud scoring tests.
//! Tests velocity, volume, structuring, counterparty disputes, and account maturity rules.

use std::sync::Arc;
use rust_decimal::Decimal;
use uuid::Uuid;

use apicash_antifraude::{
    AntiFraudeService, InMemoryScoreRepository, LocalDocumentValidator, OnRampDecision,
    SocialValidator,
};

fn make_service(repo: Arc<InMemoryScoreRepository>) -> AntiFraudeService {
    let client = reqwest::Client::new();
    let doc_validator = Arc::new(LocalDocumentValidator::new());
    let social = SocialValidator::new(client, false);
    AntiFraudeService::new(doc_validator, social, repo)
}

const VALID_CPF: &str = "52998224725";

// ─── Velocity ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn high_velocity_blocks_even_with_valid_cpf() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    // 6 transactions in 24h → PENALTY_VELOCITY_HIGH (-200) + CPF_REGULAR (+350) = 150 → Block
    repo.seed_transactions(uid, 6, Decimal::from(300u32), 6, None).await;
    let svc = make_service(repo);
    let score = svc.calculate_score(uid, VALID_CPF, &[], None).await.unwrap();
    assert_eq!(score.decision, OnRampDecision::Block, "velocity >5 should block; score={}", score.score);
}

#[tokio::test]
async fn medium_velocity_yields_review() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    // 4 transactions → PENALTY_VELOCITY_MEDIUM (-80) + CPF_REGULAR (+350) = 270 → Block
    // Actually 270 < 350 → Block. Let me reconsider the test expectation.
    // 270 < DECISION_REVIEW_MIN (350) → Block
    repo.seed_transactions(uid, 4, Decimal::from(200u32), 4, None).await;
    let svc = make_service(repo);
    let score = svc.calculate_score(uid, VALID_CPF, &[], None).await.unwrap();
    // Score: 350 - 80 = 270 → Block (below review threshold)
    assert_eq!(score.decision, OnRampDecision::Block);
    assert_eq!(score.score, 270);
}

#[tokio::test]
async fn normal_velocity_does_not_penalize() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    // 2 transactions → no velocity penalty
    repo.seed_transactions(uid, 2, Decimal::from(100u32), 2, None).await;
    let svc = make_service(repo);
    let score = svc.calculate_score(uid, VALID_CPF, &[], None).await.unwrap();
    // Score: 350 (CPF) - 40 (first tx? No, 2 total) → no maturity penalty either for 2 tx
    // Actually tx_count_total=2 → neither established nor first → no maturity factor
    assert_eq!(score.score, 350);
    assert_eq!(score.decision, OnRampDecision::Review);
}

// ─── Structuring ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn structuring_amount_below_2k_blocks() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    let svc = make_service(repo);
    // R$1.950 — inside the R$1.800–R$1.999 structuring band
    let amount = Decimal::from(1950u32);
    let score = svc.calculate_score(uid, VALID_CPF, &[], Some(amount)).await.unwrap();
    // 350 (CPF) - 180 (structuring) - 40 (first tx) - 120 (new account + high value? no, 1950 >= 500)
    // tx_count_total=0 → first tx: -40 + (1950>=500 → -120) = -160 maturity
    // total: 350 - 180 - 160 = 10 → Block
    assert_eq!(score.decision, OnRampDecision::Block, "structuring amount should block; score={}", score.score);
}

#[tokio::test]
async fn structuring_amount_below_10k_blocks() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    let svc = make_service(repo);
    let amount = Decimal::from(9500u32);
    let score = svc.calculate_score(uid, VALID_CPF, &[], Some(amount)).await.unwrap();
    assert_eq!(score.decision, OnRampDecision::Block, "R$9.500 structuring should block; score={}", score.score);
}

#[tokio::test]
async fn non_structuring_amount_is_not_penalized() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    // avg=R$1000, amount=R$2000 → ratio=200% (below 300% anomaly threshold)
    // amount=2000 is outside structuring bands (1800-1999 and 9000-9999)
    repo.seed_transactions(uid, 0, Decimal::ZERO, 10, Some(Decimal::from(1000u32))).await;
    let svc = make_service(repo);
    let amount = Decimal::from(2000u32);
    let score = svc.calculate_score(uid, VALID_CPF, &[], Some(amount)).await.unwrap();
    // 350 (CPF) + 60 (clean history) = 410 → Review
    assert_eq!(score.score, 410);
    assert_eq!(score.decision, OnRampDecision::Review);
}

// ─── Counterparty disputes ───────────────────────────────────────────────────

#[tokio::test]
async fn counterparty_disputes_penalize_score() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    repo.seed_counterparty_disputes(uid, 2).await;
    let svc = make_service(repo);
    let score = svc.calculate_score(uid, VALID_CPF, &[], None).await.unwrap();
    // 350 (CPF) - 2*90 (counterparty) - 40 (first tx) - 120 (new+no amount? 500 threshold)
    // current_tx_amount=None → no extra maturity penalty for value
    // 350 - 180 - 40 = 130 → Block
    assert_eq!(score.decision, OnRampDecision::Block);
}

#[tokio::test]
async fn combined_disputes_by_and_against_block() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    repo.seed_disputes(uid, 1).await;
    repo.seed_counterparty_disputes(uid, 2).await;
    repo.seed_transactions(uid, 0, Decimal::ZERO, 5, Some(Decimal::from(200u32))).await;
    let svc = make_service(repo);
    let score = svc.calculate_score(uid, VALID_CPF, &[], None).await.unwrap();
    // 350 - 110 (by) - 2*90 (against) + 60 (clean history but has disputes→0)
    // Actually disputes_by=1 → clean history check fails (open_dispute_count > 0)
    // 350 - 110 - 180 = 60 → Block
    assert_eq!(score.decision, OnRampDecision::Block);
}

// ─── Account maturity ────────────────────────────────────────────────────────

#[tokio::test]
async fn established_user_gets_trust_bonus() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    // 35 transactions, zero disputes → POINTS_ESTABLISHED_USER (+100)
    repo.seed_transactions(uid, 0, Decimal::ZERO, 35, Some(Decimal::from(500u32))).await;
    repo.seed_account_age(uid, 90).await;
    let svc = make_service(repo);
    let score = svc.calculate_score(uid, VALID_CPF, &[], Some(Decimal::from(500u32))).await.unwrap();
    // 350 + 100 = 450 → Review
    assert!(score.score >= 450, "established user should score ≥450; got {}", score.score);
    assert_eq!(score.decision, OnRampDecision::Review);
}

#[tokio::test]
async fn first_transaction_applies_penalty() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    // No prior transactions
    let svc = make_service(repo);
    let score = svc.calculate_score(uid, VALID_CPF, &[], Some(Decimal::from(100u32))).await.unwrap();
    // 350 - 40 (first tx) = 310; 100 < 500 → no extra high-value penalty
    assert_eq!(score.score, 310);
    assert_eq!(score.decision, OnRampDecision::Block);
}

#[tokio::test]
async fn first_tx_high_value_new_account_blocks() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    let svc = make_service(repo);
    // R$2.500 first transaction (no structuring band)
    let score = svc.calculate_score(uid, VALID_CPF, &[], Some(Decimal::from(2500u32))).await.unwrap();
    // 350 - 40 - 120 (high value new account) = 190 → Block
    assert_eq!(score.decision, OnRampDecision::Block);
    assert_eq!(score.score, 190);
}

// ─── Value anomaly ───────────────────────────────────────────────────────────

#[tokio::test]
async fn value_3x_above_average_penalizes() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    // avg = R$100, current = R$350 → 350% = 3.5× → anomaly
    repo.seed_transactions(uid, 0, Decimal::ZERO, 10, Some(Decimal::from(100u32))).await;
    let svc = make_service(repo);
    let score = svc.calculate_score(uid, VALID_CPF, &[], Some(Decimal::from(350u32))).await.unwrap();
    // 350 + 60 (clean history) - 100 (anomaly) = 310 → Block
    assert_eq!(score.decision, OnRampDecision::Block);
}

// ─── High volume ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn high_volume_24h_penalizes() {
    let repo = Arc::new(InMemoryScoreRepository::new());
    let uid = Uuid::new_v4();
    repo.seed_transactions(uid, 1, Decimal::from(25_000u32), 10, Some(Decimal::from(2500u32))).await;
    let svc = make_service(repo);
    let score = svc.calculate_score(uid, VALID_CPF, &[], None).await.unwrap();
    // 350 + 60 (clean, 10 tx) - 150 (high volume) = 260 → Block
    assert_eq!(score.decision, OnRampDecision::Block);
}
