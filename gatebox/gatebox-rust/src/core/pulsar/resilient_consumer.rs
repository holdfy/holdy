// Pulsar resilient consumer - mensageria default do projeto
use std::sync::Arc;

use futures::StreamExt;
use pulsar::SubType;

use crate::core::messaging::{MessageHandler, PaymentMessage};

use super::config::Config;

pub struct ResilientConsumer {
    config: Config,
    handler: Arc<dyn MessageHandler>,
}

impl ResilientConsumer {
    pub fn new(config: Config, handler: Arc<dyn MessageHandler>) -> Self {
        ResilientConsumer { config, handler }
    }

    pub async fn run(
        &self,
        mut cancel: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pulsar = pulsar::Pulsar::builder(&self.config.url, pulsar::TokioExecutor)
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("pulsar connect: {}", e))?;

        let mut consumer: pulsar::Consumer<Vec<u8>, _> = pulsar
            .consumer()
            .with_topic(self.config.topic_full_name.clone())
            .with_subscription(self.config.subscription_name.clone())
            .with_subscription_type(SubType::Shared)
            .with_consumer_name("gatebox-payment-consumer")
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("pulsar consumer: {}", e))?;

        tracing::info!(
            "Pulsar ResilientConsumer started on topic {}",
            self.config.topic_full_name
        );

        let handler = Arc::clone(&self.handler);
        loop {
            tokio::select! {
                _ = &mut cancel => {
                    tracing::info!("Pulsar consumer cancelled");
                    break;
                }
                msg = consumer.next() => {
                    let msg = match msg {
                        Some(Ok(m)) => m,
                        Some(Err(e)) => {
                            tracing::error!("Pulsar consumer error: {}", e);
                            continue;
                        }
                        None => break,
                    };
                    let data = msg.deserialize();
                    let payment_msg: PaymentMessage = match serde_json::from_slice(&data) {
                        Ok(m) => m,
                        Err(e) => {
                            tracing::error!("Invalid PaymentMessage: {}", e);
                            let _ = consumer.ack(&msg).await;
                            continue;
                        }
                    };
                    if let Err(e) = handler.handle(payment_msg).await {
                        tracing::error!("Handler error: {}", e);
                        let _ = consumer.nack(&msg).await;
                    } else {
                        let _ = consumer.ack(&msg).await;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
