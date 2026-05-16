//! Normalização de celular brasileiro (somente dígitos, DDI 55).

use crate::models::WhatsAppEvent;

/// Celular a partir do cartão de contacto (vCard) no evento.
#[must_use]
pub fn extract_phone_from_event(ev: &WhatsAppEvent) -> Option<String> {
    ev.contact_phone_digits
        .as_ref()
        .and_then(|d| normalize_br_mobile(d))
}

/// Cartão de contacto presente mas número inválido após normalização.
#[must_use]
pub fn contact_phone_rejected(ev: &WhatsAppEvent) -> bool {
    ev.contact_phone_digits
        .as_ref()
        .is_some_and(|raw| !raw.is_empty() && normalize_br_mobile(raw).is_none())
}

/// Remove tudo que não for dígito.
#[must_use]
pub fn digits_only(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Garante parte nacional com 11 dígitos (DDD + 9 + 8), inserindo o `9` móvel se faltar.
fn pad_national_mobile(national: &str) -> Option<String> {
    let b = national.as_bytes();
    if national.len() == 11 && b.get(2) == Some(&b'9') {
        return Some(national.to_string());
    }
    if national.len() == 10 && b.len() >= 10 {
        let ddd = &national[0..2];
        let rest = &national[2..];
        // vCard `+55 41 9999-9999` → …4199999999 (10) — falta o 9 móvel após o DDD
        if rest.starts_with('9') && rest.len() == 8 {
            return Some(format!("{ddd}9{rest}"));
        }
        if b[2] != b'9' {
            return Some(format!("{ddd}9{rest}"));
        }
    }
    None
}

/// Ajusta sequência de dígitos antes de validar (DDI, 9º dígito, vCard `+55 41 9999-9999`).
fn expand_br_digits(mut d: String) -> String {
    while d.starts_with('0') && d.len() > 1 {
        d = d[1..].to_string();
    }

    if d.starts_with("55") {
        let national = &d[2..];
        if let Some(padded) = pad_national_mobile(national) {
            return format!("55{padded}");
        }
        return d;
    }

    if let Some(padded) = pad_national_mobile(&d) {
        return format!("55{padded}");
    }
    if d.len() <= 11 {
        return format!("55{d}");
    }
    d
}

/// Normaliza para celular BR: só dígitos, com DDI `55` quando ausente.
pub fn normalize_br_mobile(raw: &str) -> Option<String> {
    let d = expand_br_digits(digits_only(raw));
    if d.is_empty() {
        return None;
    }
    if is_valid_br_mobile(&d) {
        Some(d)
    } else {
        None
    }
}

/// Celular BR: `55` + DDD (2) + 9 + 8 dígitos (13 no total).
fn is_valid_br_mobile(digits: &str) -> bool {
    if !digits.starts_with("55") || digits.len() != 13 {
        return false;
    }
    let national = &digits[2..];
    national.len() == 11
        && national.as_bytes().get(2) == Some(&b'9')
        && national.chars().all(|c| c.is_ascii_digit())
}

/// Chave de peer sempre com DDI 55 (para sessão e envio).
#[must_use]
pub fn canonical_peer_key(raw: &str) -> Option<String> {
    normalize_br_mobile(raw)
}

/// Variantes para consulta LID↔PN (com e sem DDI).
#[must_use]
pub fn peer_lookup_digit_variants(peer: &str) -> Vec<String> {
    let digits: String = peer.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return Vec::new();
    }
    let mut out = vec![digits.clone()];
    if let Some(canon) = normalize_br_mobile(&digits) {
        if canon.starts_with("55") && canon.len() == 13 {
            let national = canon[2..].to_string();
            if !out.contains(&national) {
                out.push(national);
            }
        }
        if !out.contains(&canon) {
            out.push(canon);
        }
    }
    if !digits.starts_with("55") && digits.len() == 11 && !out.contains(&format!("55{digits}")) {
        out.push(format!("55{digits}"));
    }
    out
}

/// Localiza celular na frase (máscaras, +55, blocos separados por espaço/parênteses).
pub fn extract_phone_from_text(text: &str) -> Option<String> {
    let lower = text.to_lowercase();

    // "… para <celular> …" ou "… para <celular> no valor de …"
    if let Some(pos) = lower.find(" para ") {
        let mut tail = text[pos + 6..].trim();
        if let Some(v) = tail.to_lowercase().find(" no valor") {
            tail = tail[..v].trim();
        }
        if let Some(p) = normalize_br_mobile(tail) {
            return Some(p);
        }
    }

    // "… para <celular> no valor …" (celular antes do valor)
    if let Some(vpos) = lower.find(" no valor") {
        let head = text[..vpos].trim();
        if let Some(ppos) = head.to_lowercase().rfind(" para ") {
            let mid = head[ppos + 6..].trim();
            if let Some(p) = normalize_br_mobile(mid) {
                return Some(p);
            }
        }
    }

    if let Some(p) = normalize_br_mobile(text) {
        return Some(p);
    }

    // Blocos numéricos separados (ex.: "(41) 99999-9999") — ignora sequências curtas (valor).
    let bytes = text.as_bytes();
    let mut best: Option<String> = None;
    let mut i = 0usize;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i - start < 10 {
            continue;
        }
        let chunk = &text[start..i];
        if let Some(norm) = normalize_br_mobile(chunk) {
            if best.as_ref().is_none_or(|b| norm.len() > b.len()) {
                best = Some(norm);
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_and_ddi() {
        assert_eq!(
            normalize_br_mobile("(41) 99999-9999").as_deref(),
            Some("5541999999999")
        );
        assert_eq!(
            normalize_br_mobile("41 99999-9999").as_deref(),
            Some("5541999999999")
        );
        assert_eq!(
            normalize_br_mobile("+55 (41) 99999-9999").as_deref(),
            Some("5541999999999")
        );
        assert_eq!(
            normalize_br_mobile("5541999999999").as_deref(),
            Some("5541999999999")
        );
        assert_eq!(
            normalize_br_mobile("41999999999").as_deref(),
            Some("5541999999999")
        );
    }

    #[test]
    fn ten_digit_local_without_mobile_nine() {
        assert_eq!(
            normalize_br_mobile("(41) 9999-9999").as_deref(),
            Some("5541999999999")
        );
        assert_eq!(
            normalize_br_mobile("4199999999").as_deref(),
            Some("5541999999999")
        );
    }

    #[test]
    fn vcard_twelve_digit_plus55_concatenated() {
        // +55 41 9999-9999 → dígitos colados sem o 9 móvel extra
        assert_eq!(
            normalize_br_mobile("554199999999").as_deref(),
            Some("5541999999999")
        );
    }

    #[test]
    fn invalid_short() {
        assert!(normalize_br_mobile("123").is_none());
    }

    #[test]
    fn extract_from_sentence() {
        assert_eq!(
            extract_phone_from_text("fazer um holdfy para 41999999999 no valor de 20").as_deref(),
            Some("5541999999999")
        );
    }

    #[test]
    fn extract_from_contact_event() {
        let ev = crate::models::WhatsAppEvent::with_contact_phone("5511", "mid", "41987134374");
        assert_eq!(
            extract_phone_from_event(&ev).as_deref(),
            Some("5541987134374")
        );
    }

    #[test]
    fn rejects_invalid_contact_event() {
        let ev = crate::models::WhatsAppEvent::with_contact_phone("5511", "mid", "123");
        assert!(extract_phone_from_event(&ev).is_none());
        assert!(contact_phone_rejected(&ev));
    }

    #[test]
    fn lookup_variants_include_with_and_without_ddi() {
        let v = peer_lookup_digit_variants("41999999999");
        assert!(v.contains(&"41999999999".to_string()));
        assert!(v.contains(&"5541999999999".to_string()));
    }
}
