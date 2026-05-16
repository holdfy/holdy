//! Modelos de disputa, evidência e resolução.

mod dispute;
mod evidence;
mod resolution;

pub use dispute::{Dispute, DisputeParty, DisputeStatus};
pub use evidence::{Evidence, EvidenceKind};
pub use resolution::ResolutionType;
