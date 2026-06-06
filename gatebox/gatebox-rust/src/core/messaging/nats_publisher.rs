use async_nats::jetstream;
use async_trait::async_trait;
use std::collections::HashMap;

use super::interfaces::PaymentPublisher;
use super::types::{GatewayFailureConfig, PaymentMessage};

pub struct NatsPaymentPublisher {
    context: jetstream::Context,
    subject: String,
}

impl NatsPaymentPublisher {
    pub async fn new(nats_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = async_nats::connect(nats_url).await?;
        let context = jetstream::new(client);
        let subject = "gatebox.payments".to_string();

        context
            .get_or_create_stream(jetstream::stream::Config {
                name: "GATEBOX_PAYMENTS".to_string(),
                subjects: vec!["gatebox.payments".to_string()],
                retention: jetstream::stream::RetentionPolicy::WorkQueue,
                max_age: std::time::Duration::from_secs(7 * 24 * 3600),
                ..Default::default()
            })
            .await?;

        tracing::info!(%nats_url, %subject, "nats payment publisher ready");
        Ok(Self { context, subject })
    }
}

#[async_trait]
impl PaymentPublisher for NatsPaymentPublisher {
    async fn publish(
        &self,
        payment_id: i64,
        amount: f64,
        failure_configs: Option<HashMap<String, GatewayFailureConfig>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = PaymentMessage { payment_id, amount, failure_configs };
        let payload = serde_json::to_vec(&msg)?;
        self.context
            .publish(self.subject.clone(), payload.into())
            .await?
            .await?;
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
