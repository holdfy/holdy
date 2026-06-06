//! Erros do crate de eventos.

use pulsar::error::ConsumerError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("pulsar: {0}")]
    Pulsar(#[from] pulsar::Error),

    #[error("consumer ack: {0}")]
    ConsumerAck(#[from] ConsumerError),

    #[error("handler: {0}")]
    Handler(String),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("nats: {0}")]
    Nats(String),

    #[error("serialization: {0}")]
    Serialization(String),
}
