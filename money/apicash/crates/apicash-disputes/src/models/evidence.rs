//! Evidências anexadas a uma disputa (fotos, rastreio, mensagens).

use serde::{Deserialize, Serialize};

/// Tipo de evidência enviada pelo comprador ou vendedor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Photo,
    TrackingCode,
    Message,
    Other,
}

/// Uma peça de evidência (metadados + referência opcional a URL/hash).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: EvidenceKind,
    pub description: String,
    /// URL, hash ou código externo (ex.: rastreio).
    pub reference: Option<String>,
}
