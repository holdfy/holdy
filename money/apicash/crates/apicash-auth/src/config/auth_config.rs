//! Configuração JWT e modos de autenticação (env).

use serde::{Deserialize, Serialize};

/// Configuração carregada de variáveis de ambiente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Segredo HS256 (mínimo 32 caracteres recomendado em produção).
    pub jwt_secret: String,
    pub jwt_issuer: String,
    /// TTL do access token em segundos.
    pub jwt_ttl_secs: u64,
    /// TTL do refresh token em segundos (troca por novo access).
    pub jwt_refresh_ttl_secs: u64,
    /// Se `true`, middlewares de gateway/admin aceitam qualquer requisição (apenas dev/test).
    pub auth_disabled: bool,
}

impl AuthConfig {
    /// Lê `APICASH_JWT_SECRET`, `APICASH_JWT_ISSUER`, `APICASH_JWT_TTL_SECS`, `APICASH_AUTH_DISABLED`.
    ///
    /// Sem `APICASH_JWT_SECRET`, usa segredo de desenvolvimento (log `warn!` uma vez em runtime no serviço).
    pub fn from_env() -> Self {
        Self {
            jwt_secret: match std::env::var("APICASH_JWT_SECRET") {
                Ok(s) if s.is_empty() => String::new(),
                Ok(s) => s,
                Err(_) => {
                    tracing::warn!(
                        "APICASH_JWT_SECRET not set — using insecure development default; set in production"
                    );
                    "dev_only_change_me_in_prod________".into()
                }
            },
            jwt_issuer: std::env::var("APICASH_JWT_ISSUER").unwrap_or_else(|_| "apicash".into()),
            jwt_ttl_secs: std::env::var("APICASH_JWT_TTL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
            jwt_refresh_ttl_secs: std::env::var("APICASH_JWT_REFRESH_TTL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(604_800),
            auth_disabled: std::env::var("APICASH_AUTH_DISABLED")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
        }
    }

    /// Configuração explícita (testes): auth desativado no middleware, segredo apenas para `login` em testes.
    pub fn local_dev_open() -> Self {
        Self {
            jwt_secret: "dev_test_secret_only______________".into(),
            jwt_issuer: "apicash".into(),
            jwt_ttl_secs: 3600,
            jwt_refresh_ttl_secs: 604_800,
            auth_disabled: true,
        }
    }

    #[cfg(test)]
    fn make_cfg(auth_disabled: bool, jwt_secret: &str) -> Self {
        Self {
            jwt_secret: jwt_secret.into(),
            jwt_issuer: "apicash".into(),
            jwt_ttl_secs: 3600,
            jwt_refresh_ttl_secs: 604_800,
            auth_disabled,
        }
    }

    /// Gateway público: sem segredo JWT e sem `APICASH_API_KEY` → comportamento permissivo (dev).
    pub fn gateway_legacy_open(&self) -> bool {
        self.jwt_secret.is_empty()
            && std::env::var("APICASH_API_KEY")
                .map(|k| k.is_empty())
                .unwrap_or(true)
    }

    /// Validates security-critical configuration at startup.
    ///
    /// Returns `Err(msg)` if:
    /// - `APICASH_AUTH_DISABLED=1` without `APICASH_INSECURE_DEV=1` (dev bypass)
    /// - `APICASH_JWT_SECRET` has fewer than 32 characters
    ///
    /// Set `APICASH_INSECURE_DEV=1` in dev to suppress auth_disabled check (never in production).
    pub fn validate_startup_safety(&self) -> Result<(), String> {
        let insecure_dev = std::env::var("APICASH_INSECURE_DEV")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        if self.auth_disabled && !insecure_dev {
            return Err(
                "APICASH_AUTH_DISABLED=1 is not allowed without APICASH_INSECURE_DEV=1. \
                 This flag must never be set in production. \
                 To run in dev with auth disabled, add APICASH_INSECURE_DEV=1 to money/.env."
                    .into(),
            );
        }

        if self.jwt_secret.len() < 32 {
            return Err(format!(
                "APICASH_JWT_SECRET is only {} characters. \
                 Production requires at least 32 characters of random entropy. \
                 Generate one with: openssl rand -hex 32",
                self.jwt_secret.len()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STRONG_SECRET: &str = "a_very_strong_jwt_secret_32chars!!";

    #[test]
    fn valid_config_passes() {
        let cfg = AuthConfig::make_cfg(false, STRONG_SECRET);
        assert!(cfg.validate_startup_safety().is_ok());
    }

    #[test]
    fn auth_disabled_without_insecure_dev_fails() {
        std::env::remove_var("APICASH_INSECURE_DEV");
        let cfg = AuthConfig::make_cfg(true, STRONG_SECRET);
        assert!(cfg.validate_startup_safety().is_err());
    }

    #[test]
    fn short_jwt_secret_fails() {
        let cfg = AuthConfig::make_cfg(false, "too_short");
        let result = cfg.validate_startup_safety();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("32 characters"));
    }

    #[test]
    fn exactly_32_char_secret_passes() {
        let cfg = AuthConfig::make_cfg(false, "12345678901234567890123456789012");
        assert_eq!(cfg.jwt_secret.len(), 32);
        assert!(cfg.validate_startup_safety().is_ok());
    }
}
