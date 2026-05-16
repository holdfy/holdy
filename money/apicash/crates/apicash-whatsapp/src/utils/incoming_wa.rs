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
    cm.vcard
        .as_deref()
        .and_then(|v| extract_phone_from_vcard(v))
}

/// Texto digitado ou dígitos obtidos de cartão de contato.
#[derive(Debug, Clone)]
pub enum IncomingBody {
    Text(String),
    ContactPhoneDigits(String),
}

/// Interpreta mensagem recebida (texto, um contacto ou lista de contactos).
#[must_use]
pub fn parse_incoming_message(msg: &wa::Message) -> Option<IncomingBody> {
    if let Some(t) = msg.text_content() {
        let s = t.trim();
        if !s.is_empty() {
            return Some(IncomingBody::Text(s.to_string()));
        }
    }

    let base = msg.get_base_message();

    if let Some(cm) = base.contact_message.as_ref() {
        if let Some(d) = phone_from_contact_message(cm) {
            return Some(IncomingBody::ContactPhoneDigits(d));
        }
    }

    if let Some(arr) = base.contacts_array_message.as_ref() {
        for c in &arr.contacts {
            if let Some(d) = phone_from_contact_message(c) {
                return Some(IncomingBody::ContactPhoneDigits(d));
            }
        }
    }

    None
}
