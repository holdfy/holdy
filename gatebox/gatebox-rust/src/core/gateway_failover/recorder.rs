// recordGatewaySuccess / recordGatewayError - atualiza gateway_health e gateway_error_log
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait GatewayRecorder: Send + Sync {
    async fn record_success(&self, gateway_name: &str);
    async fn record_error(
        &self,
        gateway_name: &str,
        customer_id: i64,
        transaction_id: i64,
        error_type: &str,
        error_message: &str,
    );
}

/// No-op: quando failover não está configurado
pub struct GatewayRecorderNoop;

#[async_trait]
impl GatewayRecorder for GatewayRecorderNoop {
    async fn record_success(&self, _gateway_name: &str) {}
    async fn record_error(
        &self,
        _gateway_name: &str,
        _customer_id: i64,
        _transaction_id: i64,
        _error_type: &str,
        _error_message: &str,
    ) {
    }
}

/// Implementação que persiste em gateway_health e gateway_error_log
pub struct GatewayRecorderImpl {
    pool: Arc<PgPool>,
}

impl GatewayRecorderImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GatewayRecorder for GatewayRecorderImpl {
    async fn record_success(&self, gateway_name: &str) {
        let gw = gateway_name.trim().to_lowercase();
        if gw.is_empty() {
            return;
        }
        let r = sqlx::query(
            r#"
            UPDATE gateway_health
            SET consecutive_errors = 0,
                consecutive_successes = COALESCE(consecutive_successes, 0) + 1,
                last_success_at = NOW(),
                updated_at = NOW()
            WHERE gateway_name = $1
            "#,
        )
        .bind(&gw)
        .execute(self.pool.as_ref())
        .await;
        if let Err(e) = r {
            tracing::debug!("gateway_health record_success failed for {}: {}", gw, e);
        }
    }

    async fn record_error(
        &self,
        gateway_name: &str,
        customer_id: i64,
        transaction_id: i64,
        error_type: &str,
        error_message: &str,
    ) {
        let gw = gateway_name.trim().to_lowercase();
        if gw.is_empty() {
            return;
        }
        let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO gateway_error_log (
                gateway_name, customer_id, transaction_id, error_type, error_message,
                occurred_at, expires_at
            ) VALUES ($1, $2, $3, $4, $5, NOW(), $6)
            "#,
        )
        .bind(&gw)
        .bind(customer_id)
        .bind(transaction_id)
        .bind(error_type)
        .bind(error_message)
        .bind(expires_at)
            .execute(self.pool.as_ref())
            .await
        {
            tracing::debug!("gateway_error_log insert failed for {}: {}", gw, e);
        }

        if let Err(e) = sqlx::query(
            r#"
            UPDATE gateway_health
            SET consecutive_errors = COALESCE(consecutive_errors, 0) + 1,
                last_error_at = NOW(),
                updated_at = NOW()
            WHERE gateway_name = $1
            "#,
        )
            .bind(&gw)
            .execute(self.pool.as_ref())
            .await
        {
            tracing::debug!("gateway_health record_error update failed for {}: {}", gw, e);
        }
    }
}
