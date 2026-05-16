//! Small helpers for Stellar network selection (no chain RPC calls here).

/// Well-known Stellar network names used in config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StellarNetworkKind {
    Testnet,
    Mainnet,
    Futurenet,
    Custom,
}

/// Map a lowercase network string from configuration to a [`StellarNetworkKind`].
pub fn parse_network_label(label: &str) -> StellarNetworkKind {
    match label.to_ascii_lowercase().as_str() {
        "testnet" => StellarNetworkKind::Testnet,
        "mainnet" | "pubnet" => StellarNetworkKind::Mainnet,
        "futurenet" => StellarNetworkKind::Futurenet,
        _ => StellarNetworkKind::Custom,
    }
}

/// Network passphrase for transaction signing (SEP-0023 style identifiers).
pub fn network_passphrase(kind: StellarNetworkKind) -> &'static str {
    match kind {
        StellarNetworkKind::Testnet => "Test SDF Network ; September 2015",
        StellarNetworkKind::Mainnet => "Public Global Stellar Network ; September 2015",
        StellarNetworkKind::Futurenet => "Test SDF Future Network ; October 2022",
        StellarNetworkKind::Custom => "",
    }
}

/// Default Horizon base URL for built-in networks (can be overridden in config).
pub fn default_horizon_url(kind: StellarNetworkKind) -> Option<&'static str> {
    match kind {
        StellarNetworkKind::Testnet => Some("https://horizon-testnet.stellar.org"),
        StellarNetworkKind::Mainnet => Some("https://horizon.stellar.org"),
        StellarNetworkKind::Futurenet => Some("https://horizon-futurenet.stellar.org"),
        StellarNetworkKind::Custom => None,
    }
}

/// Default Soroban RPC URL for public endpoints (illustrative; production should use dedicated nodes).
pub fn default_soroban_rpc_url(kind: StellarNetworkKind) -> Option<&'static str> {
    match kind {
        StellarNetworkKind::Testnet => Some("https://soroban-testnet.stellar.org"),
        StellarNetworkKind::Mainnet => Some("https://soroban-mainnet.stellar.org"),
        StellarNetworkKind::Futurenet => Some("https://rpc-futurenet.stellar.org"),
        StellarNetworkKind::Custom => None,
    }
}
