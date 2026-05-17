//! Política `APICASH_REQUIRE_TESTNET`: transações Stellar/Soroban devem ser reais na testnet pública.

use crate::utils::stellar::{parse_network_label, StellarNetworkKind};

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn env_nonempty(name: &str) -> bool {
    std::env::var(name)
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

/// Quando verdadeiro, escrow BRLx e lock Soroban não podem usar mocks nem fallback silencioso.
pub fn require_testnet() -> bool {
    env_truthy("APICASH_REQUIRE_TESTNET")
}

/// Valida variáveis mínimas para visibilidade no explorador da testnet.
pub fn validate_testnet_live_config() -> Result<(), Vec<String>> {
    if !require_testnet() {
        return Ok(());
    }

    let mut errs = Vec::new();

    let network = std::env::var("APICASH_STELLAR_NETWORK")
        .or_else(|_| std::env::var("STELLAR_NETWORK"))
        .unwrap_or_default();
    if parse_network_label(&network) != StellarNetworkKind::Testnet {
        errs.push(format!(
            "APICASH_STELLAR_NETWORK/STELLAR_NETWORK must be testnet (got '{network}')"
        ));
    }

    if !env_truthy("APICASH_SOROBAN_ENABLED") {
        errs.push("APICASH_SOROBAN_ENABLED=1 (custódia on-chain)".into());
    }
    if !env_truthy("APICASH_SOROBAN_STRICT") {
        errs.push("APICASH_SOROBAN_STRICT=1 (proíbe fallback mock no Soroban)".into());
    }

    for var in [
        "APICASH_SOROBAN_ESCROW_CONTRACT_ID",
        "APICASH_BRLX_TOKEN_CONTRACT_ID",
        "APICASH_STELLAR_BUYER_ADDRESS",
        "APICASH_STELLAR_SELLER_ADDRESS",
    ] {
        if !env_nonempty(var) {
            errs.push(format!("{var} must be set"));
        }
    }

    if let Ok(escrow) = std::env::var("APICASH_SOROBAN_ESCROW_CONTRACT_ID") {
        let e = escrow.to_ascii_lowercase();
        if e.contains("mock") {
            errs.push(
                "APICASH_SOROBAN_ESCROW_CONTRACT_ID cannot be a mock placeholder".into(),
            );
        }
    }

    let has_signer = env_nonempty("APICASH_SOROBAN_SOURCE_SECRET")
        || env_nonempty("APICASH_SOROBAN_BUYER_SOURCE")
        || env_nonempty("APICASH_STELLAR_SECRET_KEY");
    if !has_signer {
        errs.push(
            "APICASH_SOROBAN_SOURCE_SECRET or APICASH_SOROBAN_BUYER_SOURCE (conta testnet fundada)"
                .into(),
        );
    }

    let rpc = std::env::var("APICASH_SOROBAN_RPC_URL")
        .or_else(|_| std::env::var("STELLAR_RPC_URL"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !rpc.contains("testnet") && !rpc.contains("futurenet") {
        errs.push(format!(
            "Soroban RPC must be a public testnet endpoint (got '{rpc}')"
        ));
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

/// Falha com mensagem agregada (uso no arranque dos binários).
pub fn assert_testnet_live_config() -> Result<(), String> {
    match validate_testnet_live_config() {
        Ok(()) => Ok(()),
        Err(errs) => Err(format!(
            "APICASH_REQUIRE_TESTNET=1: configure Stellar/Soroban testnet antes de subir (transações devem aparecer no explorador):\n  - {}",
            errs.join("\n  - ")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_testnet_off_by_default() {
        std::env::remove_var("APICASH_REQUIRE_TESTNET");
        assert!(!require_testnet());
        assert!(validate_testnet_live_config().is_ok());
    }
}
