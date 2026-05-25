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

/// Aceita CPF (11 dígitos) ou CNPJ (14 dígitos). Retorna os dígitos extraídos.
pub fn parse_document(s: &str) -> Option<String> {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 11 || digits.len() == 14 {
        Some(digits)
    } else {
        None
    }
}

/// Retorna `true` se o texto parece uma PF (CPF = 11 dígitos).
pub fn is_pf_document(doc: &str) -> bool {
    doc.chars().filter(|c| c.is_ascii_digit()).count() == 11
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

/// Retorna `true` se o corpo da mensagem é uma URL HTTP/HTTPS (link de anúncio, reel, shop…).
pub fn is_product_url(s: &str) -> bool {
    let t = s.trim();
    (t.starts_with("http://") || t.starts_with("https://")) && t.len() > 12 && !t.contains('\n')
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

/// Extrai código de rastreio do corpo da mensagem.
/// Aceita: "rastrear AA123456789BR", "AA123456789BR" (sozinho), "tracking AA123..."
pub fn extract_tracking_code(body: &str) -> Option<String> {
    // Strip prefix "rastrear", "tracking", "rastreio" (case insensitive)
    let clean = body.trim();
    let text = ["rastrear ", "tracking ", "rastreio ", "rastrear:", "tracking:"]
        .iter()
        .fold(clean.to_string(), |s, p| {
            if s.to_lowercase().starts_with(p) {
                s[p.len()..].trim().to_string()
            } else {
                s
            }
        });
    // Check if remainder is a Correios code: 2 alpha + 9 digit + 2 alpha
    let t = text.trim().to_uppercase();
    if t.len() == 13
        && t.chars().take(2).all(|c| c.is_ascii_alphabetic())
        && t.chars().skip(2).take(9).all(|c| c.is_ascii_digit())
        && t.chars().skip(11).all(|c| c.is_ascii_alphabetic())
    {
        return Some(t);
    }
    // Also accept longer codes (Jadlog, etc.) that are 13-20 alphanumeric chars
    if t.len() >= 10 && t.len() <= 20 && t.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Some(t);
    }
    None
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
