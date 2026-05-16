// GatewaySelector - seleciona gateway por circuit state (CLOSED = disponível)
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait GatewaySelector: Send + Sync {
    /// Retorna o gateway disponível com menor prioridade (circuit CLOSED).
    /// Fallback para gateway_fixo se nenhum disponível ou erro.
    async fn select_gateway(&self, gateway_fallback: &str) -> Option<String>;
}

/// Implementação que consulta gateway_config + gateway_health
pub struct GatewaySelectorImpl {
    pool: Arc<PgPool>,
}

impl GatewaySelectorImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GatewaySelector for GatewaySelectorImpl {
    async fn select_gateway(&self, gateway_fallback: &str) -> Option<String> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT gc.gateway_name
            FROM gateway_config gc
            LEFT JOIN gateway_health gh ON gc.gateway_name = gh.gateway_name
            WHERE gc.enabled = true
              AND gc.supports_cash_out = true
              AND (gh.circuit_state IN ('CLOSED', 'HALF_OPEN') OR gh.circuit_state IS NULL)
            ORDER BY gc.priority ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .ok()
        .flatten();

        row.map(|r| r.0).or_else(|| {
            if gateway_fallback.is_empty() {
                None
            } else {
                Some(gateway_fallback.to_string())
            }
        })
    }
}
