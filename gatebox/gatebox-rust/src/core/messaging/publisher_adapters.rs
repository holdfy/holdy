// Adapters that implement PaymentPublisher for RabbitMQ and Pulsar ProducerPools.
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use super::interfaces::PaymentPublisher;
use super::types::{GatewayFailureConfig, PaymentMessage};

/// RabbitMQ adapter: wraps ProducerPool and implements PaymentPublisher.
/// Pool must be started before use.
pub struct RabbitMQPaymentPublisher {
    pool: Arc<crate::core::rabbitmq::ProducerPool>,
}

impl RabbitMQPaymentPublisher {
    pub fn new(pool: Arc<crate::core::rabbitmq::ProducerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentPublisher for RabbitMQPaymentPublisher {
    async fn publish(
        &self,
        payment_id: i64,
        amount: f64,
        failure_configs: Option<HashMap<String, GatewayFailureConfig>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = crate::core::rabbitmq::PublishMessage {
            payment_id,
            amount,
            failure_configs,
        };
        self.pool.publish(msg).await
    }

    fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

/// Pulsar adapter: wraps ProducerPool and implements PaymentPublisher.
/// Pool must be started before use.
pub struct PulsarPaymentPublisher {
    pool: Arc<crate::core::pulsar::ProducerPool>,
}

impl PulsarPaymentPublisher {
    pub fn new(pool: Arc<crate::core::pulsar::ProducerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentPublisher for PulsarPaymentPublisher {
    async fn publish(
        &self,
        payment_id: i64,
        amount: f64,
        failure_configs: Option<HashMap<String, GatewayFailureConfig>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = PaymentMessage {
            payment_id,
            amount,
            failure_configs,
        };
        self.pool.send(&msg).await
    }

    fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
