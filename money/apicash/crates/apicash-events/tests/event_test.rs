//! Testes de serialização dos eventos (sem broker Pulsar).

use apicash_events::models::{ApicashEvent, OrderCreatedEvent, PaymentReceivedEvent};
use apicash_shared::Money;
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

#[test]
fn roundtrip_order_created() {
    let ev = ApicashEvent::OrderCreated(OrderCreatedEvent {
        order_id: Uuid::nil(),
        buyer_id: Uuid::nil(),
        seller_id: Uuid::nil(),
        amount: Money::new(Decimal::new(1000, 2)),
        created_at: Utc::now(),
    });
    let json = serde_json::to_string(&ev).expect("serialize");
    let back: ApicashEvent = serde_json::from_str(&json).expect("deserialize");
    match back {
        ApicashEvent::OrderCreated(o) => assert_eq!(o.order_id, Uuid::nil()),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn roundtrip_payment_received() {
    let ev = ApicashEvent::PaymentReceived(PaymentReceivedEvent {
        order_id: Uuid::nil(),
        buyer_id: Uuid::nil(),
        seller_id: Uuid::nil(),
        amount: Money::new(Decimal::ONE),
        received_at: Utc::now(),
        correlation_id: Uuid::nil(),
    });
    let json = serde_json::to_string(&ev).unwrap();
    let back: ApicashEvent = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ApicashEvent::PaymentReceived(_)));
}
