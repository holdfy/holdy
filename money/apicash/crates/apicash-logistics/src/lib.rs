//! Shipping integration via Melhor Envio API.
//!
//! Melhor Envio agrega Correios + Jadlog e expõe uma API REST unificada.
//! Documentação: https://melhorenvio.com.br/developers
//!
//! Variáveis de ambiente necessárias:
//! - `MELHOR_ENVIO_TOKEN` — Bearer token OAuth2
//! - `MELHOR_ENVIO_SANDBOX` — `1` para sandbox (padrão dev)

pub mod client;
pub mod error;
pub mod service;
pub mod types;

pub use client::MelhorEnvioClient;
pub use error::LogisticsError;
pub use service::LogisticsService;
pub use types::{
    CarrierCode, PackageDimensions, ShippingAddress, ShippingLabel, ShippingQuote,
    ShippingQuoteRequest, TrackingInfo, TrackingEvent,
};
