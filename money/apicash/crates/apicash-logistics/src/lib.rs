//! Shipping integration — multi-provedor: Correios, LinkTrack, Melhor Envio.
//!
//! Variáveis de ambiente:
//! - `MELHOR_ENVIO_TOKEN`    — Bearer token OAuth2 (Melhor Envio)
//! - `MELHOR_ENVIO_SANDBOX`  — `1` para sandbox (padrão dev)
//! - `CORREIOS_USER`         — usuário Correios Business API (opcional)
//! - `CORREIOS_ACCESS_CODE`  — código cartão postagem (opcional)
//! - `LINKETRACK_USER`       — e-mail LinkTrack (opcional)
//! - `LINKETRACK_TOKEN`      — token LinkTrack (opcional)
//! - `APICASH_TRACKING_MODE` — `simulated` = LogisticaHoldFy local; omitir/`live` = cascade real
//! - `APICASH_TRACKING_SIMULATOR_URL` — URL do backend simulador (defeito: http://{MONEY_LAN_HOST}:8092)

pub mod client;
pub mod error;
pub mod service;
pub mod tracking;
pub mod types;

pub use client::MelhorEnvioClient;
pub use error::LogisticsError;
pub use service::LogisticsService;
pub use tracking::CascadingTracker;
pub use types::{
    CarrierCode, PackageDimensions, ShippingAddress, ShippingLabel, ShippingQuote,
    ShippingQuoteRequest, TrackingEvent, TrackingInfo, TrackingStatus,
};
