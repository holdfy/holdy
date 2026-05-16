//! Integração multi-device com [`whatsapp_rust`] — **este é o transporte acordado** (alternativa
//! Rust a *whatsmeow* / Baileys), **sem** depender da Cloud API da Meta para mensagens.
//!
//! ## O que falta implementar
//!
//! Após *pairing* do [`whatsapp_rust::Client`] (ou usando o módulo [`whatsapp_rust::bot`] `Bot` +
//! [`Event::Message`](whatsapp_rust::types::events::Event)), para cada mensagem de texto:
//! 1. Obter identificador do remetente (JID / E.164) compatível com [`crate::session::UserSession`].
//! 2. Corpo UTF-8 da conversa (plain text ou extração do `waproto::whatsapp::Message`).
//! 3. `tx.send(WhatsAppEvent::new(from, message_id, body)).await` com o `Sender` retornado por
//!    [`crate::service::spawn_agent`] ou [`crate::service::WhatsAppService::incoming`].
//!
//! O [`crate::handlers::MessageHandler`] e o resto do fluxo transacional **não** precisam de saber
//! se o evento veio do webhook Cloud ou do socket multi-device.

pub use whatsapp_rust::Jid;
