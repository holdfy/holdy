use super::publisher::{AnchorPublisher, PublishRequest};
use std::error::Error;

/// NoopPublisher implementa AnchorPublisher sem publicar (ANCHOR_PUBLISH_ENABLED=false ou testes).
pub struct NoopPublisher;

impl AnchorPublisher for NoopPublisher {
    fn publish_anchor_request(&self, _req: &PublishRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
