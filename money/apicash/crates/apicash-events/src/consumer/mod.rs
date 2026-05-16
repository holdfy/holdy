//! Consumers Pulsar por domínio.

mod antifraude_consumer;
mod custody_consumer;
mod release_consumer;

pub use antifraude_consumer::{run_antifraude_consumer, AntifraudeEventPort};
pub use custody_consumer::{run_custody_consumer, CustodyLockPort};
pub use release_consumer::{run_release_consumer, ReleaseEventPort};
