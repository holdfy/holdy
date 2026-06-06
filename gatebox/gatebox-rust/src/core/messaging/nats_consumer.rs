use std::sync::Arc;

use async_nats::jetstream;
use futures::StreamExt;

use super::interfaces::MessageHandler;
use super::types::PaymentMessage;

pub struct NatsConsumer {
    nats_url: String,
    handler: Arc<dyn MessageHandler>,
}

impl NatsConsumer {
    pub fn new(nats_url: impl Into<String>, handler: Arc<dyn MessageHandler>) -> Self {
        Self { nats_url: nats_url.into(), handler }
    }

    pub async fn run(
        self,
        mut shutdown: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = async_nats::connect(&self.nats_url).await?;
        let context = jetstream::new(client);

        let stream = context
            .get_or_create_stream(jetstream::stream::Config {
                name: "GATEBOX_PAYMENTS".to_string(),
                subjects: vec!["gatebox.payments".to_string()],
                retention: jetstream::stream::RetentionPolicy::WorkQueue,
                max_age: std::time::Duration::from_secs(7 * 24 * 3600),
                ..Default::default()
            })
            .await?;

        let consumer = stream
            .get_or_create_consumer(
                "gatebox-payment-consumer",
                jetstream::consumer::pull::Config {
                    durable_name: Some("gatebox-payment-consumer".to_string()),
                    ..Default::default()
                },
            )
            .await?;

        let mut messages = consumer.messages().await?;

        tracing::info!(nats_url = %self.nats_url, "nats gatebox consumer started");

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    tracing::info!("nats consumer: shutdown signal received");
                    break;
                }
                msg = messages.next() => {
                    match msg {
                        Some(Ok(msg)) => {
                            match serde_json::from_slice::<PaymentMessage>(&msg.payload) {
                                Ok(payment_msg) => {
                                    if let Err(e) = self.handler.handle(payment_msg).await {
                                        tracing::error!(error = %e, "nats: handler error");
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "nats: invalid payload, skipping");
                                }
                            }
                            let _ = msg.ack().await;
                        }
                        Some(Err(e)) => tracing::error!(error = %e, "nats: messages stream error"),
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }
}
