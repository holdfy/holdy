// Real producer: connect with lapin, publish PaymentMessage to queue
use lapin::options::*;
use lapin::types::FieldTable;
use tracing::warn;

use super::config::RabbitMQConfig;
use super::types::PaymentMessage;

pub struct Producer {
    config: RabbitMQConfig,
    connection: Option<lapin::Connection>,
    channel: Option<lapin::Channel>,
}

impl Producer {
    pub async fn new(config: RabbitMQConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let conn = lapin::Connection::connect(
            &config.uri,
            lapin::ConnectionProperties::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("rabbitmq connect: {}", e))?;
        let channel = conn.create_channel().await.map_err(|e| anyhow::anyhow!("create channel: {}", e))?;
        channel
            .queue_declare(
                &config.queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("queue_declare: {}", e))?;
        Ok(Producer {
            config,
            connection: Some(conn),
            channel: Some(channel),
        })
    }

    pub async fn publish(&self, msg: &PaymentMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ch = self.channel.as_ref().ok_or_else(|| anyhow::anyhow!("channel closed"))?;
        let body = serde_json::to_vec(msg).map_err(|e| anyhow::anyhow!("serialize: {}", e))?;
        let opts = BasicPublishOptions::default();
        let props = lapin::BasicProperties::default()
            .with_content_type("application/json".into())
            .with_delivery_mode(2) // persistent
            .with_message_id(format!("{}", msg.payment_id).into());
        ch.basic_publish(
            "",
            &self.config.queue_name,
            opts,
            &body,
            props,
        )
        .await
        .map_err(|e| {
            warn!("publish failed: {}", e);
            anyhow::anyhow!("publish: {}", e)
        })?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ch) = self.channel.take() {
            let _ = ch.close(200, "OK").await;
        }
        self.connection.take();
        Ok(())
    }
}
