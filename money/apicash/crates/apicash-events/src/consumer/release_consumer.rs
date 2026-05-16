//! Consumer de liberação final (pós-confirmação de entrega / pedido de release).

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use pulsar::{Consumer, SubType, TokioExecutor};

use crate::error::EventError;
use crate::models::{ApicashEvent, DeliveryConfirmedEvent, ReleaseRequestedEvent, SUB_RELEASE};
use crate::utils::PulsarClient;

#[async_trait]
pub trait ReleaseEventPort: Send + Sync {
    async fn on_delivery_confirmed(&self, e: DeliveryConfirmedEvent) -> Result<(), EventError>;
    async fn on_release_requested(&self, e: ReleaseRequestedEvent) -> Result<(), EventError>;
}

pub async fn run_release_consumer(
    client: &PulsarClient,
    handler: Arc<dyn ReleaseEventPort>,
) -> Result<(), EventError> {
    let topic = client.main_topic();
    let mut consumer: Consumer<ApicashEvent, TokioExecutor> = client
        .inner
        .consumer()
        .with_topic(&topic)
        .with_consumer_name("apicash-release-consumer")
        .with_subscription_type(SubType::Shared)
        .with_subscription(SUB_RELEASE)
        .build()
        .await?;

    tracing::info!(%topic, subscription = SUB_RELEASE, "release consumer started");

    while let Some(res) = consumer.next().await {
        match res {
            Ok(msg) => {
                let ev = msg.deserialize();
                match ev {
                    ApicashEvent::DeliveryConfirmed(d) => {
                        tracing::info!(order_id = %d.order_id, "release: DeliveryConfirmed");
                        handler.on_delivery_confirmed(d).await?;
                    }
                    ApicashEvent::ReleaseRequested(r) => {
                        tracing::info!(order_id = %r.order_id, "release: ReleaseRequested");
                        handler.on_release_requested(r).await?;
                    }
                    ApicashEvent::InvalidPayload(ref e) => {
                        tracing::warn!(error = %e.error, "release: invalid payload");
                    }
                    _ => {}
                }
                consumer.ack(&msg).await?;
            }
            Err(e) => tracing::error!(error = %e, "release consumer stream error"),
        }
    }

    Ok(())
}
