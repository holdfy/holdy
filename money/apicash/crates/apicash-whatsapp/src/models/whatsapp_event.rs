//! Eventos normalizados a partir do webhook Cloud API ou de bridges multi-device.

use serde::{Deserialize, Serialize};

/// Identificador do remetente (E.164 sem `+` ou JID string).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppEvent {
    pub sender_id: String,
    pub message_id: String,
    pub body: String,
    /// Telefone obtido de mensagem de contacto (vCard), só dígitos.
    #[serde(default)]
    pub contact_phone_digits: Option<String>,
    /// Nome de perfil WhatsApp do remetente (push_name do protocolo).
    #[serde(default)]
    pub push_name: Option<String>,
}

impl WhatsAppEvent {
    /// Construtor usado por testes e bridges (ex.: `whatsapp-rust`).
    pub fn new(
        sender_id: impl Into<String>,
        message_id: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            sender_id: sender_id.into(),
            message_id: message_id.into(),
            body: body.into(),
            contact_phone_digits: None,
            push_name: None,
        }
    }

    pub fn with_contact_phone(
        sender_id: impl Into<String>,
        message_id: impl Into<String>,
        digits: impl Into<String>,
    ) -> Self {
        Self {
            sender_id: sender_id.into(),
            message_id: message_id.into(),
            body: String::new(),
            contact_phone_digits: Some(digits.into()),
            push_name: None,
        }
    }

    /// Texto (ex.: intenção Holdfy + valor) e cartão de contacto na mesma mensagem.
    pub fn with_text_and_contact(
        sender_id: impl Into<String>,
        message_id: impl Into<String>,
        body: impl Into<String>,
        contact_digits: impl Into<String>,
    ) -> Self {
        Self {
            sender_id: sender_id.into(),
            message_id: message_id.into(),
            body: body.into(),
            contact_phone_digits: Some(contact_digits.into()),
            push_name: None,
        }
    }
}
