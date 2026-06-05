//! Modelos de disputa, evidência e resolução.

mod dispute;
pub mod evidence;
mod resolution;

pub use dispute::{
    AiVerdict, Dispute, DisputeParty, DisputeReason, DisputeStatus, EvidenceAnalysisResult,
};
pub use evidence::{Evidence, EvidenceKind, EvidenceParty, EvidenceRow};
pub use resolution::ResolutionType;
