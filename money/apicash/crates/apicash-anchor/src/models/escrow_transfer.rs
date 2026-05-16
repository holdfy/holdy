//! Transferência de BRLx (token) para o endereço do contrato Soroban de escrow após on-ramp.

use serde::{Deserialize, Serialize};

/// Resultado da transferência on-ledger para o contrato de custódia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowTokenTransferResult {
    pub tx_hash: String,
    pub status: String,
    /// `true` quando a feature `mock` está ativa ou transferência ainda não assinada na rede.
    pub is_mock: bool,
}
