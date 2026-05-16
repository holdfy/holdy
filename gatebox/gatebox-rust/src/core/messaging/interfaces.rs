// From app/modules/core/messaging/interfaces.go + handler.go
use async_trait::async_trait;
use std::collections::HashMap;

use super::types::{GatewayFailureConfig, PaymentMessage};

/// Publishes payment messages (implemented by rabbitmq/pulsar producer pools).
#[async_trait]
pub trait PaymentPublisher: Send + Sync {
    async fn publish(
        &self,
        payment_id: i64,
        amount: f64,
        failure_configs: Option<HashMap<String, GatewayFailureConfig>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Manages a pool of workers consuming messages (implemented by rabbitmq/pulsar worker pools).
pub trait WorkerPoolLike: Send + Sync {
    fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn get_stats(&self) -> HashMap<String, serde_json::Value>;
    fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn increment_processed(&self);
    fn increment_succeeded(&self);
    fn increment_failed(&self);
    fn increment_retried(&self);
    fn is_running(&self) -> bool;
    fn get_active_workers(&self) -> i32;
}

/// Implemented by RabbitMQ/Pulsar consumers to process payment messages.
#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, msg: PaymentMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
