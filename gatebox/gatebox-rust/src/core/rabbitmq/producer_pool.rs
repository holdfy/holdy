use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use super::config::RabbitMQConfig;
use super::producer::Producer;
use super::types::PaymentMessage;

#[derive(Debug, Clone)]
pub struct ProducerPoolConfig {
    pub num_producers: usize,
    pub batch_size: usize,
    pub confirm_timeout: Duration,
    pub buffer_size: usize,
}

impl Default for ProducerPoolConfig {
    fn default() -> Self {
        ProducerPoolConfig {
            num_producers: 10,
            batch_size: 100,
            confirm_timeout: Duration::from_secs(1),
            buffer_size: 1000,
        }
    }
}

#[derive(Debug)]
pub struct PublishMessage {
    pub payment_id: i64,
    pub amount: f64,
    pub failure_configs: Option<std::collections::HashMap<String, crate::core::messaging::GatewayFailureConfig>>,
}

impl From<PublishMessage> for PaymentMessage {
    fn from(m: PublishMessage) -> Self {
        PaymentMessage {
            payment_id: m.payment_id,
            amount: m.amount,
            failure_configs: m.failure_configs,
        }
    }
}

pub struct ProducerPool {
    _pool_config: ProducerPoolConfig,
    rb_config: RabbitMQConfig,
    producer: Arc<Mutex<Option<Producer>>>,
}

impl ProducerPool {
    pub fn new(rb_config: RabbitMQConfig, pool_config: ProducerPoolConfig) -> Self {
        ProducerPool {
            rb_config,
            _pool_config: pool_config,
            producer: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self.producer.lock().await;
        if guard.is_none() {
            let p = Producer::new(self.rb_config.clone()).await?;
            *guard = Some(p);
        }
        Ok(())
    }

    pub async fn publish(&self, msg: PublishMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self.producer.lock().await;
        let producer = guard.as_mut().ok_or_else(|| anyhow::anyhow!("ProducerPool not started"))?;
        let pm: PaymentMessage = msg.into();
        producer.publish(&pm).await
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self.producer.lock().await;
        if let Some(mut p) = guard.take() {
            let _ = p.close().await;
        }
        Ok(())
    }
}
