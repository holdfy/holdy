// Real consumer: connect with lapin, consume from queue, call MessageHandler
use futures::StreamExt;
use lapin::options::*;
use lapin::types::FieldTable;
use std::sync::Arc;
use tracing::{error, info};

use super::config::RabbitMQConfig;
use super::types::{MessageHandler, PaymentMessage};

pub struct Consumer {
    config: RabbitMQConfig,
    handler: Arc<dyn MessageHandler>,
}

impl Consumer {
    pub fn new(config: RabbitMQConfig, handler: Arc<dyn MessageHandler>) -> Self {
        Consumer { config, handler }
    }

    pub async fn run(
        &self,
        mut cancel: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = lapin::Connection::connect(
            &self.config.uri,
            lapin::ConnectionProperties::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("rabbitmq connect: {}", e))?;
        let channel = conn.create_channel().await.map_err(|e| anyhow::anyhow!("create channel: {}", e))?;
        channel
            .queue_declare(
                &self.config.queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("queue_declare: {}", e))?;
        channel
            .basic_qos(1, BasicQosOptions::default())
            .await
            .map_err(|e| anyhow::anyhow!("basic_qos: {}", e))?;
        let mut consumer = channel
            .basic_consume(
                &self.config.queue_name,
                "payment-consumer-rust",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("basic_consume: {}", e))?;
        info!("RabbitMQ consumer started on queue '{}'", self.config.queue_name);

        let handler = Arc::clone(&self.handler);
        loop {
            tokio::select! {
                _ = &mut cancel => {
                    info!("Consumer cancelled");
                    break;
                }
                delivery = consumer.next() => {
                    let d = match delivery {
                        Some(Ok(d)) => d,
                        Some(Err(e)) => {
                            error!("Consumer error: {}", e);
                            continue;
                        }
                        None => break,
                    };
                    let msg: PaymentMessage = match serde_json::from_slice(&d.data) {
                        Ok(m) => m,
                        Err(e) => {
                            error!("Invalid message body: {}", e);
                            let _ = d.ack(BasicAckOptions::default()).await;
                            continue;
                        }
                    };
                    if let Err(e) = handler.handle(msg).await {
                        error!("Handler error: {}", e);
                        let _ = d.nack(BasicNackOptions::default()).await;
                    } else {
                        let _ = d.ack(BasicAckOptions::default()).await;
                    }
                }
            }
        }
        Ok(())
    }
}
