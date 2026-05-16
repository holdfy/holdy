//! Persistência de disputas.

mod dispute_repository;

pub use dispute_repository::{
    DisputeRepository, InMemoryDisputeRepository, PostgresDisputeRepository,
};
