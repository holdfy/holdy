//! **APICash ↔ Stellar**: Anchor on/off-ramp (PIX → **BRLx**, **BRLx** → PIX), Horizon reads, and hooks for **Soroban**.
//!
//! This crate is the primary integration point with the **Stellar network**. User funds are **tokenized**
//! on-ledger (typically as **BRLx** or the configured [`config::StellarConfig::asset_code`]) while fiat
//! settlement uses the Anchor’s PIX rails. **Never** hardcode URLs or keys — use environment variables
//! via [`config::StellarConfig::from_env`] (after optional [`dotenvy::dotenv()`]).
//!
//! Product invariant: on-ramp PIX is always sent to the issuer/anchor institutional account. End-user
//! transfers (buyer→seller) happen in token rails (Stellar/Soroban), not via direct PIX to seller.
//!
//! Stellar Rust ecosystem crates (crates.io: [`stellar_rpc_client`], [`stellar_xdr`]) are re-exported for
//! downstream wiring; [`soroban_sdk`] is optional behind feature `soroban` for contract lock/unlock.

pub mod client;
pub mod config;
pub mod errors;
pub mod models;
pub mod service;

pub use config::{StellarConfig, StellarNetwork};
pub use errors::AnchorError;
pub use models::{EscrowTokenTransferResult, OffRampResponse, OnRampResponse, StellarTransaction};
pub use service::AnchorService;

pub use stellar_rpc_client;
pub use stellar_xdr;

#[cfg(feature = "soroban")]
pub use soroban_sdk;
