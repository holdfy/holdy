//! Mensageria assíncrona APICash com **Apache Pulsar** (`pulsar` / pulsar-rs).
//!
//! - [`EventProducer`] publica eventos tipados no tópico configurado.
//! - Consumers (`run_*_consumer`) filtram variantes de [`ApicashEvent`] e delegam a *ports* assíncronos.

pub mod config;
pub mod consumer;
pub mod error;
pub mod models;
pub mod producer;
pub mod utils;

pub use crate::consumer::{run_importer_consumer, run_importer_consumer_nats, ImporterPort};
pub use crate::error::EventError;
pub use crate::models::{ApicashEvent, ImportCompletedEvent, ImportRequestedEvent};
pub use crate::producer::EventProducer;
pub use crate::utils::PulsarClient;
