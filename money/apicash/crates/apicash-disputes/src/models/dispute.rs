//! Agregado de disputa.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::evidence::Evidence;
use super::resolution::ResolutionType;

/// Quem abriu a disputa no contexto do pedido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisputeParty {
    Buyer,
    Seller,
}

/// Ciclo de vida da disputa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisputeStatus {
    Open,
    UnderReview,
    Resolved,
    Closed,
}

/// Disputa de escrow: fundos permanecem travados até resolução.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub id: Uuid,
    pub order_id: Uuid,
    /// Papel de quem abriu (comprador ou vendedor).
    pub opened_by: DisputeParty,
    /// Identificador do usuário que abriu (para eventos e auditoria).
    pub opened_by_user_id: Uuid,
    pub reason: String,
    pub status: DisputeStatus,
    pub evidence: Vec<Evidence>,
    pub opened_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    /// Preenchido ao resolver; `Manual` pode não implicar movimentação de escrow.
    pub resolution_type: Option<ResolutionType>,
    pub resolution_notes: Option<String>,
}
