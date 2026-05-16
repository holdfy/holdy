//! Contratos Soroban APICash — custódia on-chain alinhada com `apicash-custody` (yield 70/10/20).
//!
//! Compilar para Wasm de deploy: `stellar contract build` na pasta deste crate.

#![no_std]

mod escrow;
mod types;
mod yield_distributor;

pub use escrow::{EscrowContract, EscrowContractClient};
pub use types::{DataKey, DisputeResolution, EscrowError, EscrowRecord, EscrowStatus};
pub use yield_distributor::{accrued_yield_simple, split_yield_pool};
