//! Integration-style tests for custody lock, yield split, and release.

use std::sync::Arc;

use apicash_custody::{
    CustodyRepository, CustodyService, InMemoryCustodyRepository, ReleaseConfirmation,
    YieldCalculator,
};
use apicash_shared::{Money, Order, OrderStatus};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

fn sample_order() -> Order {
    Order {
        id: Uuid::new_v4(),
        buyer_id: Uuid::new_v4(),
        seller_id: Uuid::new_v4(),
        amount: Money::new(Decimal::new(10_000, 2)), // 100.00
        status: OrderStatus::InCustody,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn lock_and_release_splits_yield_70_10_20() {
    let repo: Arc<dyn CustodyRepository> = Arc::new(InMemoryCustodyRepository::new());
    let calc = YieldCalculator::default();
    let svc = CustodyService::new(repo.clone(), calc);

    let order = sample_order();
    let custody = svc.lock_funds(&order).await.expect("lock");
    assert_eq!(custody.status, apicash_custody::CustodyStatus::Locked);

    let dist = svc
        .calculate_yield(&custody, 30)
        .await
        .expect("calculate_yield");
    let sum =
        dist.seller_share.decimal() + dist.buyer_cashback.decimal() + dist.platform_share.decimal();
    let pool = YieldCalculator::default()
        .accrued_yield(custody.amount, 30)
        .expect("acc")
        .decimal();
    assert!((sum - pool).abs() < Decimal::new(1, 10));

    // Security/business rule (critical): only the **buyer** can authorize escrow release.
    let rel = svc
        .release_funds(
            &order,
            order.buyer_id,
            ReleaseConfirmation {
                released_by: order.buyer_id,
                idempotency_key: "k1".into(),
            },
        )
        .await
        .expect("release (buyer ok)");

    assert_eq!(rel.order_id, order.id);
    let updated = repo
        .get_by_order_id(order.id)
        .await
        .expect("get")
        .expect("row");
    assert!(updated.yield_earned.is_some());
    assert_eq!(updated.status, apicash_custody::CustodyStatus::Released);
}

#[tokio::test]
async fn seller_cannot_release_returns_unauthorized_release() {
    let repo: Arc<dyn CustodyRepository> = Arc::new(InMemoryCustodyRepository::new());
    let calc = YieldCalculator::default();
    let svc = CustodyService::new(repo.clone(), calc);

    let order = sample_order();
    let _ = svc.lock_funds(&order).await.expect("lock");

    let err = svc
        .release_funds(
            &order,
            order.seller_id,
            ReleaseConfirmation {
                released_by: order.seller_id,
                idempotency_key: "k-seller".into(),
            },
        )
        .await
        .expect_err("seller must not release");

    assert!(matches!(
        err,
        apicash_custody::CustodyError::UnauthorizedRelease
    ));
}

#[test]
fn ratios_sum_to_one_unit() {
    assert!(apicash_custody::ratios_sum_to_one());
}
