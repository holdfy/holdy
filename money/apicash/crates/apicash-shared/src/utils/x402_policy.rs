//! Política `APICASH_X402_REQUIRED`: rotas protegidas exigem pagamento HTTP x402 (Base Sepolia USDC)
//! ou JWT válido (bypass no middleware).

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

/// Quando verdadeiro, `apicash-core` activa middleware x402 nas rotas protegidas.
pub fn require_x402() -> bool {
    env_truthy("APICASH_X402_REQUIRED")
}

/// URL do facilitator x402 (ex. `https://facilitator.x402.rs`).
pub fn facilitator_url() -> Option<String> {
    std::env::var("X402_FACILITATOR_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
}

/// Endereço EVM que recebe USDC (treasury HoldFy).
pub fn pay_to_address() -> Option<String> {
    std::env::var("X402_PAY_TO")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
}

/// Preço por request em USDC (string decimal, ex. `0.01`).
pub fn price_usdc() -> String {
    std::env::var("X402_PRICE_USDC")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "0.01".to_string())
}

/// Rede x402 suportada neste projeto (`base-sepolia` por defeito).
pub fn network_label() -> String {
    std::env::var("X402_NETWORK")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "base-sepolia".to_string())
}

/// URL pública da API (recurso no 402). Fallback para bind local.
pub fn public_base_url() -> String {
    std::env::var("APICASH_PUBLIC_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:3000".to_string())
}

pub fn validate_x402_config() -> Result<(), Vec<String>> {
    if !require_x402() {
        return Ok(());
    }

    let mut errs = Vec::new();

    if !env_nonempty("X402_FACILITATOR_URL") {
        errs.push("X402_FACILITATOR_URL must be set (ex. https://facilitator.x402.rs)".into());
    }
    if !env_nonempty("X402_PAY_TO") {
        errs.push("X402_PAY_TO must be set (EVM address on Base Sepolia)".into());
    }

    let network = network_label().to_ascii_lowercase();
    if network != "base-sepolia" && network != "base_sepolia" {
        errs.push(format!(
            "X402_NETWORK must be base-sepolia for now (got '{network}')"
        ));
    }

    let price = price_usdc();
    if price.parse::<f64>().ok().filter(|&p| p > 0.0).is_none() {
        errs.push(format!("X402_PRICE_USDC must be a positive decimal (got '{price}')"));
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

pub fn assert_x402_config() -> Result<(), String> {
    match validate_x402_config() {
        Ok(()) => Ok(()),
        Err(errs) => Err(format!(
            "APICASH_X402_REQUIRED=1: configure x402 antes de subir:\n  - {}",
            errs.join("\n  - ")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_x402_off_by_default() {
        std::env::remove_var("APICASH_X402_REQUIRED");
        assert!(!require_x402());
        assert!(validate_x402_config().is_ok());
    }
}
