//! User reputation and trust seals.
//!
//! A `UserReputation` is computed from:
//! - The user's antifraude score (0–1000)
//! - Their completed transaction count
//! - Their dispute rate (disputes / completed)
//!
//! Seals escalate: Verified → Premium → Authenticated.

pub mod reputation_model;
pub mod reputation_service;

pub use reputation_model::{ReputationSeal, UserReputation};
pub use reputation_service::ReputationService;
