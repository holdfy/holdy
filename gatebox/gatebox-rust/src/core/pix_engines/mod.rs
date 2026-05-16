// Pix engines: per-provider orchestration (from app/modules/core/pix_engines)
// Each engine wraps a gateway HTTP service for PIX flows (send, qrcode, balance).
mod sulcred_pix;
mod seventrust_pix;

pub use sulcred_pix::SulcredPixEngine;
pub use seventrust_pix::SevenTrustPixEngine;
