// Pulsar producer pool - mensageria default do projeto
use std::sync::Arc;

use tokio::sync::Mutex;

use super::config::Config;

type PulsarClient = pulsar::Pulsar<pulsar::TokioExecutor>;

pub struct ProducerPool {
    config: Config,
    client: Arc<Mutex<Option<PulsarClient>>>,
    producer: Arc<Mutex<Option<pulsar::Producer<pulsar::TokioExecutor>>>>,
}

impl ProducerPool {
    pub fn new(config: Config) -> Self {
        ProducerPool {
            config,
            client: Arc::new(Mutex::new(None)),
            producer: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut client_guard = self.client.lock().await;
        if client_guard.is_none() {
            let client = pulsar::Pulsar::builder(&self.config.url, pulsar::TokioExecutor)
                .build()
                .await
                .map_err(|e| anyhow::anyhow!("pulsar connect: {}", e))?;
            *client_guard = Some(client);
        }
        drop(client_guard);

        let mut producer_guard = self.producer.lock().await;
        if producer_guard.is_none() {
            let client = self.client.lock().await;
            let pulsar = client.as_ref().ok_or_else(|| anyhow::anyhow!("pulsar not connected"))?;
            let producer = pulsar
                .producer()
                .with_topic(self.config.topic_full_name.clone())
                .with_name("gatebox-payment-producer")
                .build()
                .await
                .map_err(|e| anyhow::anyhow!("pulsar producer: {}", e))?;
            *producer_guard = Some(producer);
        }
        tracing::info!("Pulsar ProducerPool started on topic {}", self.config.topic_full_name);
        Ok(())
    }

    pub async fn send(&self, msg: &crate::core::messaging::PaymentMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self.producer.lock().await;
        let producer = guard.as_mut().ok_or_else(|| anyhow::anyhow!("ProducerPool not started"))?;
        let payload = serde_json::to_vec(msg).map_err(|e| anyhow::anyhow!("serialize: {}", e))?;
        let message = pulsar::producer::Message {
            payload,
            ..Default::default()
        };
        producer
            .send_non_blocking(message)
            .await
            .map_err(|e| anyhow::anyhow!("pulsar send: {}", e))?
            .await
            .map_err(|e| anyhow::anyhow!("pulsar receipt: {}", e))?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut producer_guard = self.producer.lock().await;
        *producer_guard = None;
        let mut client_guard = self.client.lock().await;
        *client_guard = None;
        tracing::info!("Pulsar ProducerPool stopped");
        Ok(())
    }
}
