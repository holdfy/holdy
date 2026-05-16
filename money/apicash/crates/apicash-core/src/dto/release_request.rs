//! Custody release body.

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ReleaseRequestBody {
    pub order_id: Uuid,
    pub released_by: Uuid,
    pub idempotency_key: String,
}
