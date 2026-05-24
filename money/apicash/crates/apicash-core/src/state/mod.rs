//! Axum shared state.

mod app_state;
mod order_repository;
mod proposal_repository;

pub use app_state::{AppState, StoredOrder};
pub use order_repository::{InMemoryOrderRepository, OrderRepository, PostgresOrderRepository};
pub use proposal_repository::{InMemoryProposalRepository, ProposalRepository};
