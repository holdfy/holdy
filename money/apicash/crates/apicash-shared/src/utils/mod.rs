//! Shared helpers.

pub mod stellar;
pub mod testnet_policy;

pub use stellar::{
    default_horizon_url, default_soroban_rpc_url, network_passphrase, parse_network_label,
    StellarNetworkKind,
};
pub use testnet_policy::{
    assert_testnet_live_config, require_testnet, validate_testnet_live_config,
};
