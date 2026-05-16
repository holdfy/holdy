//! Parser unificado: intenção + valor + celular numa única mensagem.

use crate::models::WhatsAppEvent;

use super::amount::extract_amount_from_text;
use super::intent::{is_create_holdfy_intent, HoldfyIntent};
use super::phone::{extract_phone_from_event, extract_phone_from_text};

fn resolve_phone(text: &str, ev: Option<&WhatsAppEvent>) -> Option<String> {
    extract_phone_from_text(text).or_else(|| ev.and_then(extract_phone_from_event))
}

/// Resultado do parser HoldFy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHoldfyMessage {
    pub intent: HoldfyIntent,
    pub amount: Option<String>,
    pub phone: Option<String>,
}

/// Analisa mensagem completa (inclui cartão de contacto no evento).
pub fn parse_holdfy_message(text: &str, ev: Option<&WhatsAppEvent>) -> Option<ParsedHoldfyMessage> {
    if !is_create_holdfy_intent(text) {
        return None;
    }
    let phone = resolve_phone(text, ev);
    let amount = extract_amount_from_text(text);
    Some(ParsedHoldfyMessage {
        intent: HoldfyIntent::CreateHoldfy,
        amount,
        phone,
    })
}

/// Durante o fluxo guiado: extrai valor/celular mesmo sem palavra-chave holdfy.
pub fn parse_loose_fields(text: &str, ev: Option<&WhatsAppEvent>) -> (Option<String>, Option<String>) {
    let phone = resolve_phone(text, ev).or_else(|| {
        if !text.trim().is_empty() {
            super::phone::normalize_br_mobile(text.trim())
        } else {
            None
        }
    });
    let mut amount = extract_amount_from_text(text);
    if let (Some(ref a), Some(ref p)) = (&amount, &phone) {
        let ad = super::phone::digits_only(a);
        let pd = super::phone::digits_only(p);
        if ad == pd || pd.ends_with(&ad) || ad.ends_with(&pd) {
            amount = None;
        }
    }
    (amount, phone)
}

/// Próximo passo do fluxo após aplicar dados ao rascunho.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldfyCollectStep {
    AskAmount,
    AskPhone,
    Ready,
}

#[must_use]
pub fn next_collect_step(
    amount: Option<&str>,
    phone: Option<&str>,
) -> HoldfyCollectStep {
    match (amount, phone) {
        (Some(_), Some(_)) => HoldfyCollectStep::Ready,
        (Some(_), None) => HoldfyCollectStep::AskPhone,
        (None, Some(_)) => HoldfyCollectStep::AskAmount,
        (None, None) => HoldfyCollectStep::AskAmount,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_sentence_phone_first() {
        let p = parse_holdfy_message(
            "fazer um holdfy para 41999999999 no valor de 20",
            None,
        )
        .expect("parsed");
        assert_eq!(p.amount.as_deref(), Some("20"));
        assert_eq!(p.phone.as_deref(), Some("5541999999999"));
    }

    #[test]
    fn full_sentence_amount_first() {
        let p = parse_holdfy_message(
            "fazer um holdfy de 20 para 41999999999",
            None,
        )
        .expect("parsed");
        assert_eq!(p.amount.as_deref(), Some("20"));
        assert_eq!(p.phone.as_deref(), Some("5541999999999"));
    }

    #[test]
    fn masked_phone() {
        let p = parse_holdfy_message(
            "quero fazer um holdfy de R$ 20,00 para (41) 99999-9999",
            None,
        )
        .expect("parsed");
        assert_eq!(p.amount.as_deref(), Some("20"));
        assert_eq!(p.phone.as_deref(), Some("5541999999999"));
    }

    #[test]
    fn intent_only() {
        let p = parse_holdfy_message("fazer um holdfy", None).expect("parsed");
        assert!(p.amount.is_none());
        assert!(p.phone.is_none());
    }

    #[test]
    fn only_amount_in_intent() {
        let p = parse_holdfy_message("fazer um holdfy de 20", None).expect("parsed");
        assert_eq!(p.amount.as_deref(), Some("20"));
        assert!(p.phone.is_none());
    }

    #[test]
    fn only_phone_in_intent() {
        let p = parse_holdfy_message("fazer um holdfy para 41999999999", None).expect("parsed");
        assert!(p.amount.is_none());
        assert_eq!(p.phone.as_deref(), Some("5541999999999"));
    }

    #[test]
    fn intent_with_amount_and_contact_card() {
        let ev = crate::models::WhatsAppEvent::with_text_and_contact(
            "5511",
            "mid",
            "fazer um holdfy de 20",
            "41999999999",
        );
        let p = parse_holdfy_message("fazer um holdfy de 20", Some(&ev)).expect("parsed");
        assert_eq!(p.amount.as_deref(), Some("20"));
        assert_eq!(p.phone.as_deref(), Some("5541999999999"));
        assert_eq!(
            next_collect_step(p.amount.as_deref(), p.phone.as_deref()),
            HoldfyCollectStep::Ready
        );
    }

    #[test]
    fn contact_card_only_while_collecting_phone() {
        let ev = crate::models::WhatsAppEvent::with_contact_phone("5511", "mid", "41999999999");
        let (amt, phone) = parse_loose_fields("", Some(&ev));
        assert!(amt.is_none());
        assert_eq!(phone.as_deref(), Some("5541999999999"));
    }
}
