//! Eventos normalizados a partir do webhook Cloud API ou de bridges multi-device.

use serde::{Deserialize, Serialize};

/// Tipo de mídia anexada à mensagem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Image,
    Video,
    Audio,
    Document,
}

impl MediaKind {
    pub fn to_evidence_kind(&self) -> &'static str {
        match self {
            Self::Image    => "photo",
            Self::Video    => "video",
            Self::Audio    => "other",
            Self::Document => "other",
        }
    }

    pub fn default_ext(&self) -> &'static str {
        match self {
            Self::Image    => "jpg",
            Self::Video    => "mp4",
            Self::Audio    => "ogg",
            Self::Document => "bin",
        }
    }
}

/// Referência a mídia recebida via Cloud API (download via Graph API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudMediaRef {
    pub kind:      MediaKind,
    /// ID para `GET https://graph.facebook.com/v20.0/{media_id}` → download URL.
    pub media_id:  String,
    pub mime_type: Option<String>,
    pub caption:   Option<String>,
    pub sha256:    Option<String>,
}

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
    /// Mídia anexada (foto, vídeo, áudio) — Cloud API popula `cloud_media_id`.
    #[serde(default)]
    pub media: Option<CloudMediaRef>,
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
            media: None,
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
            media: None,
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
            media: None,
        }
    }
}
