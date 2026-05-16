//! Extração e normalização de valores monetários em texto livre.

use rust_decimal::Decimal;

/// Valor normalizado como string decimal (`20` ou `20.5`).
pub fn normalize_amount_decimal(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let cleaned = t
        .trim_start_matches("r$")
        .trim_start_matches("R$")
        .trim()
        .replace(',', ".");
    let cleaned = cleaned.trim_end_matches("reais").trim();
    if cleaned.is_empty() {
        return None;
    }
    let d = Decimal::from_str_exact(&cleaned)
        .or_else(|_| cleaned.parse::<Decimal>())
        .ok()?;
    if d <= Decimal::ZERO {
        return None;
    }
    Some(d.round_dp(2).normalize().to_string())
}

/// Extrai o primeiro token numérico monetário em `tail` (ex.: `20,00 para` → `20.00`).
fn extract_decimal_token(tail: &str) -> Option<String> {
    let t = tail.trim();
    let mut end = 0usize;
    for (i, c) in t.char_indices() {
        if c.is_ascii_digit() || c == ',' || c == '.' {
            end = i + c.len_utf8();
        } else if end > 0 {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    normalize_amount_decimal(&t[..end])
}

/// Procura valor em frases com marcadores comuns (R$, reais, valor de, de X para).
pub fn extract_amount_from_text(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for marker in [
        "no valor de",
        "valor de",
        "valor ",
        "r$",
        "reais",
    ] {
        if let Some(pos) = lower.find(marker) {
            let tail = &text[pos + marker.len()..];
            if let Some(a) = extract_decimal_token(tail) {
                return Some(a);
            }
        }
    }
    // Padrão "de 20 para" / "de R$ 20,00 para"
    if let Some(pos) = lower.find(" de ") {
        let after = &text[pos + 4..];
        if let Some(para_pos) = after.to_lowercase().find(" para ") {
            let mid = &after[..para_pos];
            if let Some(a) = extract_decimal_token(mid) {
                return Some(a);
            }
        }
    }
    // "de 20" no fim (ex.: "fazer um holdfy de 20")
    if let Some(pos) = lower.rfind(" de ") {
        let tail = text[pos + 4..].trim();
        if !tail.to_lowercase().contains(" para ") {
            if let Some(a) = extract_decimal_token(tail) {
                return Some(a);
            }
        }
    }
    // Mensagem só com número monetário — não confundir com celular (10–13 dígitos).
    if looks_like_phone_digits(text) {
        return None;
    }
    normalize_amount_decimal(text).or_else(|| extract_decimal_token(text))
}

/// Sequência só de dígitos com tamanho típico de telefone BR (sem R$, vírgula, etc.).
fn looks_like_phone_digits(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() || t.contains(',') || t.contains('.') {
        return false;
    }
    let lower = t.to_lowercase();
    if lower.contains("r$") || lower.contains("reais") || lower.contains("valor") {
        return false;
    }
    let d: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    (10..=13).contains(&d.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(extract_amount_from_text("20").as_deref(), Some("20"));
        assert_eq!(extract_amount_from_text("20.00").as_deref(), Some("20"));
        assert_eq!(extract_amount_from_text("20,00").as_deref(), Some("20"));
        assert_eq!(extract_amount_from_text("R$ 20").as_deref(), Some("20"));
        assert_eq!(extract_amount_from_text("R$ 20,00").as_deref(), Some("20"));
        assert_eq!(extract_amount_from_text("20 reais").as_deref(), Some("20"));
        assert_eq!(
            extract_amount_from_text("no valor de 20").as_deref(),
            Some("20")
        );
        assert_eq!(
            extract_amount_from_text("fazer um holdfy de 20 para 41999999999").as_deref(),
            Some("20")
        );
        assert!(extract_amount_from_text("41988153959").is_none());
        assert!(extract_amount_from_text("5541988153959").is_none());
    }
}
