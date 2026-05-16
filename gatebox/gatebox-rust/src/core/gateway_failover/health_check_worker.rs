// Health check worker - verifica gateways OPEN/HALF_OPEN periodicamente
use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::info;

/// Health check worker: a cada intervalo, para gateways em OPEN,
/// após circuit_open_duration_minutes, atualiza para HALF_OPEN (permite retry).
pub struct HealthCheckWorker {
    _pool: Arc<PgPool>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl HealthCheckWorker {
    pub fn new(pool: Arc<PgPool>, interval_secs: u64) -> Self {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        let pool_clone = Arc::clone(&pool);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Health Check Worker stopped");
                        break;
                    }
                    _ = ticker.tick() => {
                        if let Err(e) = perform_health_checks(pool_clone.as_ref()).await {
                            tracing::debug!("Health check: {}", e);
                        }
                    }
                }
            }
        });

        info!("Health Check Worker started (interval={}s)", interval_secs);
        Self {
            _pool: pool,
            shutdown_tx: Some(shutdown_tx),
        }
    }
}

impl Drop for HealthCheckWorker {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
    }
}

async fn perform_health_checks(pool: &PgPool) -> Result<(), String> {
    // Buscar circuit_open_duration_minutes (default 10)
    let cooldown_min: i64 = sqlx::query_scalar(
        r#"SELECT COALESCE(CAST(config_value AS INT), 10) FROM gateway_failover_config WHERE config_key = 'circuit_open_duration_minutes' LIMIT 1"#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or(10);

    // Para gateways OPEN, se circuit_opened_at + cooldown passou, mover para HALF_OPEN
    let r = sqlx::query(
        r#"
        UPDATE gateway_health
        SET circuit_state = 'HALF_OPEN',
            circuit_half_opened_at = NOW(),
            consecutive_successes = 0,
            updated_at = NOW()
        WHERE circuit_state = 'OPEN'
          AND (circuit_opened_at IS NULL OR circuit_opened_at < NOW() - ($1 || ' minutes')::INTERVAL)
        "#,
    )
    .bind(cooldown_min)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    if r.rows_affected() > 0 {
        info!("Health check: {} gateway(s) moved OPEN -> HALF_OPEN", r.rows_affected());
    }

    Ok(())
}
