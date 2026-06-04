//! Persistência de disputas.

mod dispute_repository;
mod evidence_repository;

pub use dispute_repository::{
    DisputeRepository, InMemoryDisputeRepository, PostgresDisputeRepository,
};
pub use evidence_repository::{
    EvidenceRepository, InMemoryEvidenceRepository, PostgresEvidenceRepository,
};
