//! End-user identity (minimal profile for shared types).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Registered platform user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub phone_e164: Option<String>,
    pub created_at: DateTime<Utc>,
}
