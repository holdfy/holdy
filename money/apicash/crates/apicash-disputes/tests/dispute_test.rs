//! Testes de abertura, custódia `Disputed` e resolução.

use std::sync::Arc;

use apicash_custody::{
    CustodyRepository, CustodyService, CustodyStatus, InMemoryCustodyRepository, YieldCalculator,
};
use apicash_disputes::models::{DisputeParty, DisputeStatus, Evidence, EvidenceKind};
use apicash_disputes::repository::{DisputeRepository, InMemoryDisputeRepository};
use apicash_disputes::service::NoopDisputeEventSink;
use apicash_disputes::{DisputeService, ResolutionType};
use apicash_shared::{Money, Order, OrderStatus};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

fn sample_order() -> Order {
    Order {
        id: Uuid::new_v4(),
        buyer_id: Uuid::new_v4(),
        seller_id: Uuid::new_v4(),
        amount: Money::new(Decimal::new(5_000, 2)),
        status: OrderStatus::InCustody,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn open_dispute_marks_custody_disputed_and_persists() {
    let custody_repo: Arc<dyn CustodyRepository> = Arc::new(InMemoryCustodyRepository::new());
    let custody = Arc::new(CustodyService::new(
        custody_repo.clone(),
        YieldCalculator::default(),
    ));

    let order = sample_order();
    custody.lock_funds(&order).await.expect("lock");

    let dr: Arc<dyn DisputeRepository> = Arc::new(InMemoryDisputeRepository::new());
    let events = Arc::new(NoopDisputeEventSink);
    let svc = DisputeService::new(dr.clone(), custody.clone(), events, Default::default());

    let ev = vec![Evidence {
        kind: EvidenceKind::Message,
        description: "item not received".into(),
        reference: None,
    }];

    let d = svc
        .open_dispute(
            order.id,
            DisputeParty::Buyer,
            order.buyer_id,
            "not delivered".into(),
            ev,
        )
        .await
        .expect("open");

    assert_eq!(d.status, DisputeStatus::Open);
    let c = custody_repo
        .get_by_order_id(order.id)
        .await
        .expect("get")
        .expect("custody");
    assert_eq!(c.status, CustodyStatus::Disputed);
}

#[tokio::test]
async fn resolve_release_releases_custody() {
    let custody_repo: Arc<dyn CustodyRepository> = Arc::new(InMemoryCustodyRepository::new());
    let custody = Arc::new(CustodyService::new(
        custody_repo.clone(),
        YieldCalculator::default(),
    ));
    let order = sample_order();
    custody.lock_funds(&order).await.expect("lock");

    let dr: Arc<dyn DisputeRepository> = Arc::new(InMemoryDisputeRepository::new());
    let svc = DisputeService::new(
        dr,
        custody.clone(),
        Arc::new(NoopDisputeEventSink),
        Default::default(),
    );

    let d = svc
        .open_dispute(
            order.id,
            DisputeParty::Buyer,
            order.buyer_id,
            "defect".into(),
            vec![],
        )
        .await
        .expect("open");

    svc.resolve_dispute(
        d.id,
        ResolutionType::ReleaseToSeller,
        Some("seller wins".into()),
    )
    .await
    .expect("resolve");

    let c = custody_repo
        .get_by_order_id(order.id)
        .await
        .expect("get")
        .expect("custody");
    assert_eq!(c.status, CustodyStatus::Released);
}

#[tokio::test]
async fn manual_resolution_does_not_release() {
    let custody_repo: Arc<dyn CustodyRepository> = Arc::new(InMemoryCustodyRepository::new());
    let custody = Arc::new(CustodyService::new(
        custody_repo.clone(),
        YieldCalculator::default(),
    ));
    let order = sample_order();
    custody.lock_funds(&order).await.expect("lock");

    let dr: Arc<dyn DisputeRepository> = Arc::new(InMemoryDisputeRepository::new());
    let svc = DisputeService::new(
        dr,
        custody.clone(),
        Arc::new(NoopDisputeEventSink),
        Default::default(),
    );

    let d = svc
        .open_dispute(
            order.id,
            DisputeParty::Seller,
            order.seller_id,
            "buyer claim".into(),
            vec![],
        )
        .await
        .expect("open");

    svc.resolve_dispute(d.id, ResolutionType::Manual, None)
        .await
        .expect("resolve");

    let c = custody_repo
        .get_by_order_id(order.id)
        .await
        .expect("get")
        .expect("custody");
    assert_eq!(c.status, CustodyStatus::Disputed);
}
