//! Consumer que reforça atualização de score após movimentações.

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use pulsar::{Consumer, SubType, TokioExecutor};

use crate::error::EventError;
use crate::models::{ApicashEvent, ScoreCalculatedEvent, TransactionRecordedEvent, SUB_ANTIFRAUDE};
use crate::utils::PulsarClient;

#[async_trait]
pub trait AntifraudeEventPort: Send + Sync {
    async fn on_transaction_recorded(&self, e: TransactionRecordedEvent) -> Result<(), EventError>;
    async fn on_score_calculated(&self, e: ScoreCalculatedEvent) -> Result<(), EventError>;
}

pub async fn run_antifraude_consumer(
    client: &PulsarClient,
    handler: Arc<dyn AntifraudeEventPort>,
) -> Result<(), EventError> {
    let topic = client.main_topic();
    let mut consumer: Consumer<ApicashEvent, TokioExecutor> = client
        .inner
        .consumer()
        .with_topic(&topic)
        .with_consumer_name("apicash-antifraude-consumer")
        .with_subscription_type(SubType::Shared)
        .with_subscription(SUB_ANTIFRAUDE)
        .build()
        .await?;

    tracing::info!(%topic, subscription = SUB_ANTIFRAUDE, "antifraude consumer started");

    while let Some(res) = consumer.next().await {
        match res {
            Ok(msg) => {
                let ev = msg.deserialize();
                match ev {
                    ApicashEvent::TransactionRecorded(t) => {
                        tracing::info!(reference = %t.reference, "antifraude: TransactionRecorded");
                        handler.on_transaction_recorded(t).await?;
                    }
                    ApicashEvent::ScoreCalculated(s) => {
                        tracing::info!(user_id = %s.user_id, "antifraude: ScoreCalculated");
                        handler.on_score_calculated(s).await?;
                    }
                    ApicashEvent::InvalidPayload(ref e) => {
                        tracing::warn!(error = %e.error, "antifraude: invalid payload");
                    }
                    _ => {}
                }
                consumer.ack(&msg).await?;
            }
            Err(e) => tracing::error!(error = %e, "antifraude consumer stream error"),
        }
    }

    Ok(())
}
