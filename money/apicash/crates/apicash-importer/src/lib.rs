//! Universal product importer.
//!
//! Pipeline de extração em cascata (primeiro sucesso vence):
//! 1. `JsonLdExtractor`  — `schema.org/Product` JSON-LD
//! 2. `OpenGraphExtractor` — `og:title` / `og:image` / `og:description`
//! 3. `MercadoLivreExtractor` — API oficial `api.mercadolibre.com/items/{id}`
//! 4. `LlmExtractor` — fallback: envia HTML ao Claude API

pub mod error;
pub mod extractors;
pub mod service;
pub mod types;

pub use error::ImporterError;
pub use service::ImporterService;
pub use types::ProductDraft;
