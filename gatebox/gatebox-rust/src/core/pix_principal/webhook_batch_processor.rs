// Webhook batch processor - agrupa updates (20 webhooks/batch) como no Go
use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::info;

#[derive(Debug, Clone)]
pub struct WebhookUpdate {
    pub transaction_id: i64,
    pub status_id: i64,
    pub gateway: String,
    pub msg_error: String,
}

/// WebhookBatchProcessor acumula webhooks e faz batch UPDATEs no PostgreSQL
pub struct WebhookBatchProcessor {
    tx: mpsc::Sender<WebhookUpdate>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl WebhookBatchProcessor {
    pub fn new(
        pool: Arc<PgPool>,
        batch_size: usize,
        flush_interval: Duration,
    ) -> Self {
        let (update_tx, mut update_rx) = mpsc::channel::<WebhookUpdate>(batch_size * 10);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        let pool_clone = Arc::clone(&pool);
        tokio::spawn(async move {
            let mut batch: Vec<WebhookUpdate> = Vec::with_capacity(batch_size);
            let mut ticker = interval(flush_interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        if !batch.is_empty() {
                            process_batch(pool_clone.as_ref(), &batch).await;
                        }
                        info!("Webhook Batch Processor stopped");
                        break;
                    }
                    Some(update) = update_rx.recv() => {
                        batch.push(update);
                        if batch.len() >= batch_size {
                            process_batch(pool_clone.as_ref(), &batch).await;
                            batch.clear();
                        }
                    }
                    _ = ticker.tick() => {
                        if !batch.is_empty() {
                            process_batch(pool_clone.as_ref(), &batch).await;
                            batch.clear();
                        }
                    }
                }
            }
        });

        info!(
            "Webhook Batch Processor started: batch_size={}, flush_interval={:?}",
            batch_size, flush_interval
        );

        Self {
            tx: update_tx,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    pub fn queue_update(&self, transaction_id: i64, status_id: i64, gateway: &str, msg_error: &str) {
        let _ = self.tx.try_send(WebhookUpdate {
            transaction_id,
            status_id,
            gateway: gateway.to_string(),
            msg_error: msg_error.to_string(),
        });
    }
}

impl Drop for WebhookBatchProcessor {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
    }
}

async fn process_batch(pool: &PgPool, batch: &[WebhookUpdate]) {
    if batch.is_empty() {
        return;
    }

    let start = std::time::Instant::now();
    let mut total_affected: i64 = 0;

    let (success_ids, success_gateways): (Vec<i64>, Vec<String>) = batch
        .iter()
        .filter(|u| u.msg_error.is_empty())
        .map(|u| (u.transaction_id, u.gateway.clone()))
        .unzip();

    if !success_ids.is_empty() {
        let r = sqlx::query(
            r#"
            UPDATE transaction t
            SET status_transaction_id = 4, msg_error = '', gateway = u.gateway
            FROM UNNEST($1::bigint[], $2::text[]) AS u(id, gateway)
            WHERE t.id = u.id
            "#,
        )
        .bind(&success_ids)
        .bind(&success_gateways)
        .execute(pool)
        .await;

        if let Ok(res) = r {
            total_affected += res.rows_affected() as i64;
        } else if let Err(e) = r {
            tracing::error!("Batch UPDATE (success) failed: {}", e);
        }
    }

    for u in batch.iter().filter(|u| !u.msg_error.is_empty()) {
        let r = sqlx::query(
            r#"
            UPDATE transaction
            SET status_transaction_id = $1, msg_error = $2, gateway = $4
            WHERE id = $3
            "#,
        )
        .bind(u.status_id)
        .bind(&u.msg_error)
        .bind(u.transaction_id)
        .bind(&u.gateway)
        .execute(pool)
        .await;

        if let Ok(res) = r {
            total_affected += res.rows_affected() as i64;
        } else if let Err(e) = r {
            tracing::error!("UPDATE (error) failed for tx={}: {}", u.transaction_id, e);
        }
    }

    let elapsed = start.elapsed();
    info!(
        "Batch UPDATE: {} webhooks, {} affected, elapsed={:?}",
        batch.len(),
        total_affected,
        elapsed
    );
}
