//! Extração de texto ou telefone (contact / vCard) de mensagens WA protobuf.

use wacore::proto_helpers::MessageExt;
use waproto::whatsapp as wa;

/// Extrai número do vCard (prioriza `waid=`; senão maior sequência 10–15 dígitos).
pub fn extract_phone_from_vcard(vcard: &str) -> Option<String> {
    for raw_line in vcard.lines() {
        let line = raw_line.trim();
        if let Some(idx) = line.to_ascii_lowercase().find("waid=") {
            let rest = &line[idx + 5..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if digits.len() >= 10 {
                return Some(digits);
            }
        }
    }

    let mut best = String::new();
    let mut cur = String::new();
    for c in vcard.chars() {
        if c.is_ascii_digit() {
            cur.push(c);
        } else {
            if cur.len() > best.len() {
                best = cur.clone();
            }
            cur.clear();
        }
    }
    if cur.len() > best.len() {
        best = cur;
    }
    if (10..=15).contains(&best.len()) {
        Some(best)
    } else {
        None
    }
}

fn phone_from_contact_message(cm: &wa::message::ContactMessage) -> Option<String> {
    cm.vcard.as_deref().and_then(|v| {
        extract_phone_from_vcard(v)
            .and_then(|digits| crate::handlers::holdfy::normalize_br_mobile(&digits))
    })
}

/// Texto digitado ou dígitos obtidos de cartão de contato.
#[derive(Debug, Clone)]
pub enum IncomingBody {
    Text(String),
    ContactPhoneDigits(String),
    /// Legenda + cartão de contacto (ex.: "fazer um holdfy de 20" + vCard).
    TextAndContact { text: String, digits: String },
}

fn contact_digits_from_message(msg: &wa::Message) -> Option<String> {
    let base = msg.get_base_message();
    if let Some(cm) = base.contact_message.as_ref() {
        if let Some(d) = phone_from_contact_message(cm) {
            return Some(d);
        }
    }
    if let Some(arr) = base.contacts_array_message.as_ref() {
        for c in &arr.contacts {
            if let Some(d) = phone_from_contact_message(c) {
                return Some(d);
            }
        }
    }
    None
}

/// Interpreta mensagem recebida (texto, um contacto ou lista de contactos).
#[must_use]
pub fn parse_incoming_message(msg: &wa::Message) -> Option<IncomingBody> {
    let text = msg
        .text_content()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string);
    let contact = contact_digits_from_message(msg);

    match (text, contact) {
        (Some(t), Some(d)) => Some(IncomingBody::TextAndContact { text: t, digits: d }),
        (Some(t), None) => Some(IncomingBody::Text(t)),
        (None, Some(d)) => Some(IncomingBody::ContactPhoneDigits(d)),
        (None, None) => None,
    }
}
