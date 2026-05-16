//! Consumer que reage a `PaymentReceived` e aciona custódia (`lock_funds`).

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use pulsar::{Consumer, SubType, TokioExecutor};

use crate::error::EventError;
use crate::models::{ApicashEvent, PaymentReceivedEvent, SUB_CUSTODY};
use crate::utils::PulsarClient;

/// Port injetável: implementação típica delega a [`apicash_custody::CustodyService::lock_funds`].
#[async_trait]
pub trait CustodyLockPort: Send + Sync {
    async fn lock_funds(&self, order: &apicash_shared::Order) -> Result<(), EventError>;
}

/// Inicia loop de consumo: processa apenas [`ApicashEvent::PaymentReceived`].
pub async fn run_custody_consumer(
    client: &PulsarClient,
    handler: Arc<dyn CustodyLockPort>,
) -> Result<(), EventError> {
    let topic = client.main_topic();
    let mut consumer: Consumer<ApicashEvent, TokioExecutor> = client
        .inner
        .consumer()
        .with_topic(&topic)
        .with_consumer_name("apicash-custody-consumer")
        .with_subscription_type(SubType::Shared)
        .with_subscription(SUB_CUSTODY)
        .build()
        .await?;

    tracing::info!(%topic, subscription = SUB_CUSTODY, "custody consumer started");

    while let Some(res) = consumer.next().await {
        match res {
            Ok(msg) => {
                let ev = msg.deserialize();
                match ev {
                    ApicashEvent::PaymentReceived(p) => {
                        tracing::info!(order_id = %p.order_id, "custody: PaymentReceived");
                        handle_payment(&handler, p).await?;
                    }
                    ApicashEvent::InvalidPayload(ref e) => {
                        tracing::warn!(error = %e.error, "custody: skip invalid payload");
                    }
                    _ => {
                        tracing::trace!("custody: ignoring event variant");
                    }
                }
                consumer.ack(&msg).await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "custody consumer stream error");
            }
        }
    }

    Ok(())
}

async fn handle_payment(
    handler: &Arc<dyn CustodyLockPort>,
    p: PaymentReceivedEvent,
) -> Result<(), EventError> {
    let order = p.to_order_pending();
    handler.lock_funds(&order).await
}
