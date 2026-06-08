//! Job de rastreio proativo: polling a cada 30 min + notificação WhatsApp ao comprador.
//!
//! Ativado automaticamente em `spawn_agent` quando DATABASE_URL está configurado.
//! Intervalo configurável via `TRACKING_MONITOR_INTERVAL_SECS` (padrão: 1800).

use std::sync::Arc;
use std::time::Duration;

use apicash_logistics::{tracking::status_label, CascadingTracker};
use sqlx::{PgPool, Row};
use tracing::{info, warn};
use uuid::Uuid;

use crate::outbound::Outbound;
use crate::utils::message_templates;

pub struct TrackingMonitor {
    pool: PgPool,
    tracker: CascadingTracker,
    outbound: Arc<Outbound>,
}

impl TrackingMonitor {
    pub fn new(pool: PgPool, tracker: CascadingTracker, outbound: Arc<Outbound>) -> Self {
        Self { pool, tracker, outbound }
    }

    /// Spawna tokio task que chama `poll_once` no intervalo configurado.
    pub fn spawn(self) {
        tokio::spawn(async move {
            let interval_secs = std::env::var("TRACKING_MONITOR_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(30 * 60);
            info!(interval_secs, "tracking_monitor: iniciado");
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                if let Err(e) = self.poll_once().await {
                    warn!(error = %e, "tracking_monitor: poll falhou");
                }
            }
        });
    }

    async fn poll_once(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            r#"
            SELECT id, order_id, tracking_code, buyer_peer, seller_peer, last_status
            FROM order_tracking_status
            WHERE last_status NOT IN ('delivered', 'returned')
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        info!(count = rows.len(), "tracking_monitor: verificando rastreios ativos");

        for row in rows {
            let id: Uuid = row.try_get("id")?;
            let order_id: Uuid = row.try_get("order_id")?;
            let code: String = row.try_get("tracking_code")?;
            let buyer_peer: String = row.try_get("buyer_peer")?;
            let seller_peer: String = row.try_get("seller_peer")?;
            let last_status: String = row.try_get("last_status")?;

            match self.tracker.track(&code).await {
                Ok(info) => {
                    let new_status = format!("{:?}", info.current_status).to_lowercase();
                    if new_status == last_status {
                        continue;
                    }

                    let description = info
                        .events
                        .first()
                        .map(|e| e.description.clone())
                        .unwrap_or_default();

                    sqlx::query(
                        r#"
                        UPDATE order_tracking_status
                        SET last_status = $1,
                            last_event_description = $2,
                            notified_at = NOW(),
                            updated_at = NOW()
                        WHERE id = $3
                        "#,
                    )
                    .bind(&new_status)
                    .bind(if description.is_empty() {
                        None::<&str>
                    } else {
                        Some(description.as_str())
                    })
                    .bind(id)
                    .execute(&self.pool)
                    .await?;

                    let label = status_label(&info.current_status);
                    let is_delivered = matches!(
                        info.current_status,
                        apicash_logistics::TrackingStatus::Delivered
                    );

                    if is_delivered {
                        // Busca o valor do pedido para exibir na mensagem de confirmação.
                        let amount_opt: Option<String> = sqlx::query_scalar::<_, String>(
                            "SELECT ROUND(amount, 2)::text FROM orders WHERE id = $1",
                        )
                        .bind(order_id)
                        .fetch_optional(&self.pool)
                        .await
                        .ok()
                        .flatten();

                        // Comprador recebe mensagem interativa com botões Confirmar / Disputa.
                        let confirm_body = message_templates::tracking_delivered_ask_confirm(
                            &order_id,
                            amount_opt.as_deref(),
                            &code,
                        );
                        self.outbound
                            .send_interactive_confirm_receipt(&buyer_peer, &confirm_body)
                            .await;

                        // Vendedor recebe aviso de entrega com contexto do próximo passo.
                        if !seller_peer.is_empty() {
                            let order_short = format!("{:.8}", order_id);
                            self.outbound
                                .send_text(
                                    &seller_peer,
                                    &message_templates::tracking_delivered_seller_await(
                                        &code,
                                        &order_short,
                                    ),
                                )
                                .await;
                        }
                    } else {
                        // Status não-entregue: atualização genérica ao comprador.
                        let mut msg = format!(
                            "📦 *Atualização do seu pedido* (#{:.8})\n\nRastreio: `{}`\nStatus: *{}*",
                            order_id, code, label
                        );
                        if !description.is_empty() {
                            msg.push_str(&format!("\n_{}_", description));
                        }

                        self.outbound.send_text(&buyer_peer, &msg).await;

                        // Vendedor recebe apenas retorno, devolução ou problema.
                        let notify_seller = matches!(
                            info.current_status,
                            apicash_logistics::TrackingStatus::ReturnInProgress
                                | apicash_logistics::TrackingStatus::Returned
                                | apicash_logistics::TrackingStatus::Exception
                        );
                        if notify_seller && !seller_peer.is_empty() {
                            self.outbound.send_text(&seller_peer, &msg).await;
                        }
                    }

                    info!(
                        order_id = %order_id,
                        code,
                        old = %last_status,
                        new = %new_status,
                        delivered = is_delivered,
                        "tracking_monitor: notificado"
                    );
                }
                Err(e) => {
                    warn!(code, error = %e, "tracking_monitor: falha ao rastrear (ignorando)");
                }
            }
        }
        Ok(())
    }
}

/// Insere ou atualiza código de rastreio associado a um pedido.
/// Chamado pelo message_handler quando o vendedor envia o código via WhatsApp.
pub async fn upsert_tracking(
    pool: &PgPool,
    order_id: Uuid,
    tracking_code: &str,
    buyer_peer: &str,
    seller_peer: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO order_tracking_status (order_id, tracking_code, buyer_peer, seller_peer, last_status)
        VALUES ($1, $2, $3, $4, 'unknown')
        ON CONFLICT (order_id, tracking_code) DO UPDATE
            SET buyer_peer  = EXCLUDED.buyer_peer,
                seller_peer = EXCLUDED.seller_peer,
                updated_at  = NOW()
        "#,
    )
    .bind(order_id)
    .bind(tracking_code)
    .bind(buyer_peer)
    .bind(seller_peer)
    .execute(pool)
    .await?;
    Ok(())
}
