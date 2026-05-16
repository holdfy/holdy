//! Lógica do fluxo conversacional de pedido (sem efeitos colaterais de rede).

use rust_decimal::Decimal;

use crate::models::WhatsAppEvent;
use crate::session::OrderFlowState;

/// Normaliza texto do usuário (minúsculas, trim, sem pontuação final).
pub fn normalize_cmd(s: &str) -> String {
    let mut t = s.trim().to_lowercase();
    while t.ends_with(['.', '!', '?', ',']) {
        t.pop();
    }
    t
}

pub fn is_new_order(cmd: &str) -> bool {
    crate::handlers::holdfy::is_create_holdfy_intent(cmd)
}

pub fn is_help(cmd: &str) -> bool {
    matches!(
        normalize_cmd(cmd).as_str(),
        "ajuda" | "help" | "menu" | "/ajuda"
    )
}

pub fn is_cancel(cmd: &str) -> bool {
    matches!(
        normalize_cmd(cmd).as_str(),
        "cancelar"
            | "cancel"
            | "/cancelar"
            | "cancelar_fluxo"
            | "cancelar pedido"
            | "cancelar_pedido"
    )
}

pub fn is_skip(cmd: &str) -> bool {
    matches!(
        normalize_cmd(cmd).as_str(),
        "pular" | "skip" | "sem links" | "nao tenho" | "não tenho"
    )
}

/// Intent to confirm delivery (not final).
pub fn is_confirm_receipt_intent(cmd: &str) -> bool {
    matches!(
        normalize_cmd(cmd).as_str(),
        "confirmar"
            | "recebi"
            | "recebido"
            | "recebida"
            | "eu recebi"
            | "recebemos"
            | "confirmar recebimento"
    )
}

/// Final, explicit confirmation required to release escrow.
pub fn is_confirm_receipt_final(cmd: &str) -> bool {
    matches!(
        normalize_cmd(cmd).as_str(),
        "confirmar recebimento"
            | "confirmar_recebimento"
            | "confirmarrecebimento"
            | "confirmar recebimento."
    )
}

pub fn is_confirm_order_final(cmd: &str) -> bool {
    matches!(
        normalize_cmd(cmd).as_str(),
        "confirmar pedido"
            | "confirmar_pedido"
            | "confirmar pagamento"
            | "confirmar_pagamento"
            | "confirmar_pedido."
    )
}

/// Comprador (B) aceita a proposta de valor antes de gerar PIX.
pub fn is_accept_proposal(cmd: &str) -> bool {
    matches!(
        normalize_cmd(cmd).as_str(),
        "aceito"
            | "aceito!"
            | "aceite"
            | "aceitar"
            | "sim"
            | "confirmo"
            | "pode gerar"
            | "gera o pix"
            | "gera pix"
    )
}

pub fn is_reject_proposal(cmd: &str) -> bool {
    matches!(
        normalize_cmd(cmd).as_str(),
        "nao" | "não" | "recuso" | "nao aceito" | "não aceito"
    )
}

pub fn is_paid(cmd: &str) -> bool {
    matches!(
        normalize_cmd(cmd).as_str(),
        "pagamento feito"
            | "confirmo pagamento"
            | "confirmei pagamento"
            | "confirmo o pagamento"
            | "confirmei o pagamento"
            | "pix feito"
            | "ja paguei"
            | "já paguei"
            | "paguei"
            | "ja_paguei"
            | "já_paguei"
            | "pago"
            | "ja confirmei o pagamento"
            | "já confirmei o pagamento"
    )
}

pub fn is_dispute(cmd: &str) -> bool {
    matches!(
        normalize_cmd(cmd).as_str(),
        "disputa" | "abrir disputa" | "reclamação"
    )
}

/// Valor monetário como string decimal normalizada (sem `f64`).
pub fn parse_amount(s: &str) -> Option<String> {
    crate::handlers::holdfy::extract_amount_from_text(s)
        .or_else(|| {
            let t = s.trim().replace(',', ".");
            if t.is_empty() {
                return None;
            }
            let d = Decimal::from_str_exact(&t)
                .or_else(|_| t.parse::<Decimal>())
                .ok()?;
            if d <= Decimal::ZERO {
                return None;
            }
            Some(d.round_dp(2).normalize().to_string())
        })
}

pub fn parse_cpf(s: &str) -> Option<String> {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 11 {
        Some(digits)
    } else {
        None
    }
}

/// Descrição do pedido (texto livre, truncada).
pub fn parse_description(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let mut out = t.to_string();
    if out.len() > 500 {
        out.truncate(500);
    }
    Some(out)
}

pub fn parse_social_links(s: &str) -> Vec<String> {
    s.split(&[',', '\n'][..])
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

/// Telefone do comprador (B): normalização BR (DDI 55, só dígitos).
#[must_use]
pub fn parse_phone_peer_key(raw: &str) -> Option<String> {
    crate::handlers::holdfy::normalize_br_mobile(raw)
}

/// Resolve número do comprador a partir de texto ou cartão de contacto.
#[must_use]
pub fn resolve_counterparty_peer(ev: &WhatsAppEvent) -> Option<String> {
    if let Some(ref d) = ev.contact_phone_digits {
        if let Some(p) = parse_phone_peer_key(d) {
            return Some(p);
        }
    }
    parse_phone_peer_key(ev.body.trim())
}

/// Normaliza peer da sessão para comparar com telefone (só dígitos).
#[must_use]
pub fn peer_digits(peer_id: &str) -> String {
    peer_id.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Mesmo número WhatsApp (PN), para bloquear vendedor = comprador no mesmo chat.
#[must_use]
pub fn peers_same_phone(a: &str, b: &str) -> bool {
    let da = peer_digits(a);
    let db = peer_digits(b);
    !da.is_empty() && da == db
}

/// Próximo estado após comando global (ex.: disputa).
pub fn try_dispute(current: &OrderFlowState, cmd: &str) -> Option<OrderFlowState> {
    if !is_dispute(cmd) {
        return None;
    }
    match current {
        OrderFlowState::AwaitingPayment { order_id, .. }
        | OrderFlowState::AwaitingConfirmation { order_id, .. } => {
            Some(OrderFlowState::DisputeHint {
                order_id: *order_id,
            })
        }
        _ => None,
    }
}
