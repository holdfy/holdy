//! Persistence traits for scores.

pub mod score_repository;

pub use score_repository::{InMemoryScoreRepository, PostgresScoreRepository, ScoreRepository};
