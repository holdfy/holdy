//! Helpers to avoid logging sensitive PII (like full phone numbers).

/// Masks a WhatsApp peer identifier for logs.
///
/// - Keeps only last 4 digits when possible.
/// - Avoids emitting the full phone/JID in logs.
#[must_use]
pub fn mask_whatsapp_peer(peer: &str) -> String {
    let digits: String = peer.chars().filter(|c| c.is_ascii_digit()).collect();
    let tail = if digits.len() >= 4 {
        &digits[digits.len() - 4..]
    } else if peer.len() >= 4 {
        &peer[peer.len() - 4..]
    } else {
        ""
    };
    if tail.is_empty() {
        "***".into()
    } else {
        format!("***{tail}")
    }
}
