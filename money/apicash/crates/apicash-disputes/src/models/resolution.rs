//! Tipos de resolução e status de workflow.

use serde::{Deserialize, Serialize};

/// Resultado aguardado após análise (espelhado em política Soroban no futuro).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionType {
    RefundBuyer,
    ReleaseToSeller,
    /// Divisão acordada — liquidação detalhada fica para camada de custódia/contrato.
    Split,
    /// Escalonamento manual (sem liberação automática de escrow).
    Manual,
}
