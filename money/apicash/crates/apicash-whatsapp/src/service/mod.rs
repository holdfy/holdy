//! Serviço WhatsApp (`whatsapp-rust` + webhook Cloud opcional).

pub mod multidevice;
pub mod whatsapp_service;

pub use whatsapp_service::{
    notification_to_events, run_health_only_server, run_webhook_server, spawn_agent, AgentConfig,
    WaTransport, WebhookState, WhatsAppService,
};
