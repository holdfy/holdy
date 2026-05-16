//! Testes do parser HoldFy (intenção, valor, celular).

use apicash_whatsapp::handlers::holdfy::{
    extract_amount_from_text, extract_phone_from_text, is_create_holdfy_intent,
    next_collect_step, parse_holdfy_message, parse_loose_fields, HoldfyCollectStep,
};
use apicash_whatsapp::handlers::holdfy::phone::{
    contact_phone_rejected, extract_phone_from_event, normalize_br_mobile,
};
use apicash_whatsapp::models::WhatsAppEvent;

#[test]
fn intent_simple_and_synonyms() {
    assert!(is_create_holdfy_intent("fazer um holdfy"));
    assert!(is_create_holdfy_intent("quero fazer um holdfy"));
    assert!(is_create_holdfy_intent("criar holdfy"));
    assert!(is_create_holdfy_intent("gerar cobrança holdfy"));
    assert!(is_create_holdfy_intent("holdfy"));
    assert!(!is_create_holdfy_intent("olá"));
}

#[test]
fn amount_first_phrase() {
    let p = parse_holdfy_message("fazer um holdfy de 20 para 41999999999", None).unwrap();
    assert_eq!(p.amount.as_deref(), Some("20"));
    assert_eq!(p.phone.as_deref(), Some("5541999999999"));
}

#[test]
fn phone_first_phrase() {
    let p =
        parse_holdfy_message("fazer um holdfy para 41999999999 no valor de 20", None).unwrap();
    assert_eq!(p.amount.as_deref(), Some("20"));
    assert_eq!(p.phone.as_deref(), Some("5541999999999"));
}

#[test]
fn masked_phone_and_currency() {
    let p = parse_holdfy_message(
        "faça um holdfy de R$ 20,00 para (41) 99999-9999",
        None,
    )
    .unwrap();
    assert_eq!(p.amount.as_deref(), Some("20"));
    assert_eq!(p.phone.as_deref(), Some("5541999999999"));
}

#[test]
fn phone_plus55() {
    let p = parse_holdfy_message(
        "quero fazer um holdfy para +55 (41) 99999-9999 no valor de 20",
        None,
    )
    .unwrap();
    assert_eq!(p.phone.as_deref(), Some("5541999999999"));
}

#[test]
fn incomplete_only_amount() {
    let p = parse_holdfy_message("fazer um holdfy de 20", None).unwrap();
    assert_eq!(p.amount.as_deref(), Some("20"));
    assert!(p.phone.is_none());
    assert_eq!(
        next_collect_step(p.amount.as_deref(), p.phone.as_deref()),
        HoldfyCollectStep::AskPhone
    );
}

#[test]
fn incomplete_only_phone() {
    let p = parse_holdfy_message("fazer um holdfy para 41999999999", None).unwrap();
    assert!(p.amount.is_none());
    assert_eq!(p.phone.as_deref(), Some("5541999999999"));
    assert_eq!(
        next_collect_step(p.amount.as_deref(), p.phone.as_deref()),
        HoldfyCollectStep::AskAmount
    );
}

#[test]
fn context_merge_across_messages() {
    let (a1, p1) = parse_loose_fields("20", None);
    assert_eq!(a1.as_deref(), Some("20"));
    assert!(p1.is_none());
    let (a2, p2) = parse_loose_fields("41999999999", None);
    assert!(a2.is_none());
    assert_eq!(p2.as_deref(), Some("5541999999999"));
    assert_eq!(
        next_collect_step(a1.as_deref().or(a2.as_deref()), p2.as_deref()),
        HoldfyCollectStep::Ready
    );
}

#[test]
fn invalid_phone_short() {
    assert!(normalize_br_mobile("123").is_none());
    assert!(extract_phone_from_text("fazer um holdfy de 20 para 123").is_none());
}

#[test]
fn amount_formats() {
    assert_eq!(extract_amount_from_text("R$ 20,00").as_deref(), Some("20"));
    assert_eq!(extract_amount_from_text("50 reais").as_deref(), Some("50"));
    assert_eq!(extract_amount_from_text("valor de 100").as_deref(), Some("100"));
}

#[test]
fn holdfy_with_contact_card_and_amount() {
    let ev = WhatsAppEvent::with_text_and_contact(
        "5511",
        "mid",
        "quero fazer um holdfy de R$ 20,00",
        "5541999999999",
    );
    let p = parse_holdfy_message("quero fazer um holdfy de R$ 20,00", Some(&ev)).unwrap();
    assert_eq!(p.amount.as_deref(), Some("20"));
    assert_eq!(p.phone.as_deref(), Some("5541999999999"));
    assert_eq!(
        next_collect_step(p.amount.as_deref(), p.phone.as_deref()),
        HoldfyCollectStep::Ready
    );
}

#[test]
fn contact_card_after_amount_in_conversation() {
    let ev = WhatsAppEvent::with_contact_phone("5511", "mid", "41999999999");
    let (a1, _) = parse_loose_fields("fazer um holdfy de 20", None);
    let (_, p2) = parse_loose_fields("", Some(&ev));
    assert_eq!(a1.as_deref(), Some("20"));
    assert_eq!(p2.as_deref(), Some("5541999999999"));
    assert_eq!(
        next_collect_step(a1.as_deref(), p2.as_deref()),
        HoldfyCollectStep::Ready
    );
}

#[test]
fn contact_card_vcard_waid_normalized() {
    let ev = WhatsAppEvent::with_contact_phone("5511", "mid", "5541987134374");
    assert_eq!(
        extract_phone_from_event(&ev).as_deref(),
        Some("5541987134374")
    );
}

#[test]
fn ten_digits_without_ddi_or_mobile_nine() {
    assert_eq!(
        normalize_br_mobile("4199999999").as_deref(),
        Some("5541999999999")
    );
    assert_eq!(
        normalize_br_mobile("554199999999").as_deref(),
        Some("5541999999999")
    );
}

#[test]
fn invalid_contact_card() {
    let ev = WhatsAppEvent::with_contact_phone("5511", "mid", "123");
    assert!(extract_phone_from_event(&ev).is_none());
    assert!(contact_phone_rejected(&ev));
}
