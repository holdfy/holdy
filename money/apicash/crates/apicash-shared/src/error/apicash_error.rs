//! Top-level error type for cross-crate boundaries.

use std::io;

use config::ConfigError;
use serde_json::Error as SerdeJsonError;
use thiserror::Error;
use uuid::Error as UuidError;

/// Unified error enum for APICash services and libraries.
#[derive(Debug, Error)]
pub enum ApiCashError {
    /// Invalid user input or domain rule violation.
    #[error("validation: {0}")]
    Validation(String),

    /// Configuration could not be loaded or parsed.
    #[error("configuration: {0}")]
    Config(#[from] ConfigError),

    /// Serialization / deserialization failure.
    #[error("serialization: {0}")]
    Serialization(#[from] SerdeJsonError),

    /// Identifier parsing failure.
    #[error("uuid: {0}")]
    Uuid(#[from] UuidError),

    /// IO error (files, streams).
    #[error("io: {0}")]
    Io(#[from] io::Error),

    /// Monetary parse or arithmetic boundary.
    #[error("money: {0}")]
    Money(String),

    /// Resource not found (generic key).
    #[error("not found: {0}")]
    NotFound(String),

    /// Conflict with current state (e.g. duplicate idempotency key).
    #[error("conflict: {0}")]
    Conflict(String),

    /// External dependency failure (HTTP, chain RPC, messaging).
    #[error("external: {0}")]
    External(String),

    /// Authentication / authorization failure.
    #[error("auth: {0}")]
    Auth(String),

    /// Only the buyer may confirm delivery and authorize escrow release.
    #[error("Apenas o comprador pode confirmar o recebimento e liberar o pagamento.")]
    UnauthorizedRelease,

    /// Stellar / Anchor specific failure.
    #[error("stellar: {0}")]
    Stellar(String),

    /// Pulsar / messaging failure.
    #[error("pulsar: {0}")]
    Pulsar(String),

    /// WhatsApp integration failure.
    #[error("whatsapp: {0}")]
    WhatsApp(String),

    /// Anti-fraud or scoring subsystem failure.
    #[error("antifraude: {0}")]
    AntiFraud(String),

    /// Catch-all internal error with context.
    #[error("internal: {0}")]
    Internal(String),
}

impl ApiCashError {
    /// Attach validation context.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Attach external dependency context.
    pub fn external(msg: impl Into<String>) -> Self {
        Self::External(msg.into())
    }
}
