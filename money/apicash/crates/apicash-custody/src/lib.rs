//! **Custody & yield** for APICash: lock principal in escrow, accrue configurable monthly yield (simulated
//! daily), and distribute **yield only** with a 70 / 10 / 20 split on release.
//!
//! ## Stellar / Soroban (future)
//!
//! Principal and yield movements will be mirrored by **Soroban smart contracts** on Stellar: this crate
//! keeps the business rules and amounts in [`rust_decimal`] / [`apicash_shared::Money`] so they can be
//! reproduced in contract logic or compared against on-chain state from [`stellar-rpc-client`] (often
//! referred to as the ecosystem “Stellar Rust SDK”) once the `stellar-prep` Cargo feature is enabled.

pub mod errors;
pub mod models;
pub mod repository;
pub mod service;
mod soroban_bridge;

/// Yield accrual and 70/10/20 split (`yield` is a reserved keyword, so the path stays `src/yield/`).
#[path = "yield/mod.rs"]
mod yield_logic;
pub use yield_logic::{ratios_sum_to_one, split_yield_pool, YieldCalculator};

pub use errors::CustodyError;
pub use models::{Custody, CustodyStatus, ReleaseConfirmation, ReleaseResult, YieldDistribution};
pub use repository::{CustodyRepository, InMemoryCustodyRepository, PostgresCustodyRepository};
pub use service::CustodyService;
pub use soroban_bridge::{
    custody_bridge_from_env, order_key_from_uuid, LockInvokeParams, MockSorobanBridge,
    SorobanCustodyBridge, SorobanDeployOutcome, SorobanLockOutcome,
};
