//! **HoldFy Agent** — bot conversacional no WhatsApp.
//!
//! **Transporte alinhado:** [`whatsapp_rust`](https://crates.io/crates/whatsapp-rust) (multi-device em Rust, mesma família que *whatsmeow* em Go), **não** a Cloud API da Meta como caminho principal.
//! O servidor webhook + tipos Cloud existem como **auxiliar** até o *bridge* multi-device → [`WhatsAppEvent`](crate::models::WhatsAppEvent) estar completo.
//!
//! A orquestração de negócio (pedido, PIX, custódia) é feita via HTTP contra [`apicash_core::create_router`] (`apicash-core`).

pub mod conversation_store;
pub mod nfse_client;
pub mod wa_contact_store;
pub mod tracking_monitor;
pub mod core_api;
pub mod handlers;
pub mod models;
pub mod payment_notify;
pub mod outbound;
pub mod service;
pub mod session;
pub mod utils;
pub mod wa_multidevice;
pub mod wa_peer;

pub use crate::core_api::{CoreApiClient, CoreApiError, CreateOrderResponse, PixPaymentResponse};
pub use crate::models::WhatsAppEvent;
pub use crate::payment_notify::PaymentNotifyRegistry;
pub use crate::outbound::Outbound;
pub use crate::service::{
    notification_to_events, run_health_only_server, run_webhook_server, spawn_agent, AgentConfig,
    WaTransport, WebhookState, WhatsAppService,
};

/// Re-export da API principal (mesmo workspace) para deploy combinado.
pub use apicash_core::{create_router, AppState};

/// Tipos de custódia e antifraude usados no mesmo fluxo de negócio.
pub use apicash_antifraude::OnRampDecision;
pub use apicash_custody::ReleaseConfirmation;
pub use apicash_shared::Money;
