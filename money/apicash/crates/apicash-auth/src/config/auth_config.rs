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

    /// Gateway público: sem segredo JWT e sem `APICASH_API_KEY` → comportamento permissivo (dev).
    pub fn gateway_legacy_open(&self) -> bool {
        self.jwt_secret.is_empty()
            && std::env::var("APICASH_API_KEY")
                .map(|k| k.is_empty())
                .unwrap_or(true)
    }
}
