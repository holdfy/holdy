//! APICash core HTTP API: Axum gateway orchestrating custody, anchor, and anti-fraud services.
//!
//! Run the binary `apicash-core` or embed [`create_router`] in tests.
//!
//! Observability & audit: sensitive actions (order creation, delivery confirmation, fund release)
//! emit structured `tracing` logs with `user_id`, `order_id`, `action`, `success`, and `timestamp`,
//! enabling end-to-end attribution ("who did what") across services.

pub mod config;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod repository;
pub mod router;
pub mod state;

/// Re-export for callers that will wire Postgres later.
pub type PgPool = sqlx::PgPool;

pub use crate::error::ApiError;
pub use crate::router::create_router;
pub use crate::state::AppState;
