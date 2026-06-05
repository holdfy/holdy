//! **Disputas APICash** — fluxo de reclamações com escrow travado (Soroban/Stellar) até resolução.
//!
//! - [`DisputeService`] orquestra repositório, [`apicash_custody::CustodyService`] e eventos [`apicash_events`].
//! - Abertura: `CustodyStatus::Disputed` + evento `DisputeOpened`.
//! - Resolução: liberação via custódia quando [`crate::models::ResolutionType`] não for [`Manual`](crate::models::ResolutionType::Manual).

pub mod error;
pub mod handlers;
pub mod image_store;
pub mod models;
pub mod openai_client;
pub mod repository;
pub mod service;
pub mod utils;

pub use crate::models::{
    AiVerdict, Dispute, DisputeParty, DisputeReason, DisputeStatus, EvidenceAnalysisResult,
    EvidenceKind, EvidenceParty, EvidenceRow, ResolutionType,
};
pub use service::DisputeService;
