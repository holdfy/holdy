//! Testes do fluxo conversacional (sem WhatsApp nem HTTP).

use std::str::FromStr;

use apicash_whatsapp::handlers::order_flow;
use apicash_whatsapp::models::WhatsAppEvent;
use apicash_whatsapp::utils::incoming_wa;
use rust_decimal::Decimal;

#[test]
fn new_order_commands() {
    assert!(order_flow::is_new_order("novo pedido"));
    assert!(order_flow::is_new_order("fazer um holdfy"));
    assert!(order_flow::is_new_order("gerar cobrança holdfy"));
    assert!(!order_flow::is_new_order("oi"));
}

#[test]
fn accept_and_reject_proposal() {
    assert!(order_flow::is_accept_proposal("Aceito"));
    assert!(order_flow::is_accept_proposal("aceito."));
    assert!(order_flow::is_accept_proposal("SIM"));
    assert!(order_flow::is_accept_proposal("gera pix"));
    assert!(!order_flow::is_accept_proposal("talvez"));
    assert!(order_flow::is_reject_proposal("não"));
    assert!(order_flow::is_reject_proposal("recuso"));
    assert!(!order_flow::is_reject_proposal("sim"));
}

#[test]
fn confirm_and_dispute() {
    assert!(order_flow::is_confirm_receipt_intent("recebi"));
    assert!(order_flow::is_confirm_receipt_intent("confirmar"));
    assert!(order_flow::is_confirm_receipt_final(
        "CONFIRMAR RECEBIMENTO"
    ));
    assert!(order_flow::is_confirm_receipt_final(
        "CONFIRMAR_RECEBIMENTO"
    ));
    assert!(order_flow::is_confirm_order_final("CONFIRMAR_PEDIDO"));
    assert!(order_flow::is_paid("JA_PAGUEI"));
    assert!(order_flow::is_paid("pagamento feito"));
    assert!(order_flow::is_paid("confirmei o pagamento"));
    assert!(order_flow::is_dispute("disputa"));
}

#[test]
fn parse_amount_and_cpf() {
    let s = order_flow::parse_amount("100,50").expect("amount");
    assert_eq!(
        Decimal::from_str(&s).unwrap(),
        Decimal::from_str_exact("100.50").unwrap()
    );
    assert_eq!(
        order_flow::parse_cpf("123.456.789-09"),
        Some("12345678909".into())
    );
    assert!(order_flow::parse_cpf("123").is_none());
}

#[test]
fn parse_order_id_from_gatebox_qr_pipe() {
    use apicash_whatsapp::payment_notify::PaymentNotifyRegistry;
    let id = uuid::Uuid::parse_str("8e94736e-1f09-4d02-892c-5990fbfa4d70").unwrap();
    let qr = format!("GATEBOXRUST:QR|order_{id}|13.00");
    assert_eq!(
        PaymentNotifyRegistry::parse_order_id_from_reference(&qr),
        Some(id)
    );
}

#[test]
fn peers_same_phone_detects_identical_pn() {
    assert!(order_flow::peers_same_phone(
        "5541987134374",
        "5541987134374"
    ));
    assert!(order_flow::peers_same_phone(
        "+55 41 98713-4374",
        "5541987134374"
    ));
    assert!(!order_flow::peers_same_phone(
        "5541987134374",
        "554187233621"
    ));
}

#[test]
fn parse_phone_peer_key_br() {
    assert_eq!(
        order_flow::parse_phone_peer_key("+55 (41) 98713-4374").as_deref(),
        Some("5541987134374")
    );
    assert_eq!(
        order_flow::parse_phone_peer_key("5541987134374").as_deref(),
        Some("5541987134374")
    );
    assert_eq!(
        order_flow::parse_phone_peer_key("41987134374").as_deref(),
        Some("5541987134374")
    );
    assert!(order_flow::parse_phone_peer_key("123").is_none());
}

#[test]
fn resolve_counterparty_prefers_contact_field() {
    let ev = WhatsAppEvent::with_contact_phone("5511", "mid", "5541987134374");
    assert_eq!(
        order_flow::resolve_counterparty_peer(&ev).as_deref(),
        Some("5541987134374")
    );
}

#[test]
fn extract_phone_from_vcard_waid() {
    let v = "BEGIN:VCARD\nVERSION:3.0\nN:;\nTEL;waid=5541987134374:+55 41 98713-4374\nEND:VCARD";
    assert_eq!(
        incoming_wa::extract_phone_from_vcard(v).as_deref(),
        Some("5541987134374")
    );
}

#[test]
fn holdfy_merge_contact_when_asking_phone() {
    use apicash_whatsapp::handlers::holdfy::{next_collect_step, parse_loose_fields, HoldfyCollectStep};

    let (a, p) = parse_loose_fields("fazer um holdfy de 20", None);
    assert_eq!(a.as_deref(), Some("20"));
    assert!(p.is_none());

    let ev = WhatsAppEvent::with_contact_phone("5511", "mid", "41999999999");
    let (_, p2) = parse_loose_fields("", Some(&ev));
    assert_eq!(p2.as_deref(), Some("5541999999999"));
    assert_eq!(
        next_collect_step(a.as_deref(), p2.as_deref()),
        HoldfyCollectStep::Ready
    );
}
