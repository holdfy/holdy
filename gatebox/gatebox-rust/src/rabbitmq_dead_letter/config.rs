// POC: RabbitMQ dead letter / hospital queue config (from app/modules/core/rabbitmq/config.go)
use std::time::Duration;

pub const QUEUE_NAME_PAYMENT: &str = "payment-queue";
pub const QUEUE_NAME_DLX: &str = "payment-queue-dlx";
pub const QUEUE_NAME_HOSPITAL: &str = "payment-queue-hospital";
pub const EXCHANGE_NAME_DLX: &str = "payment-exchange-dlx";
pub const EXCHANGE_NAME_HOSPITAL: &str = "payment-exchange-hospital";

pub const DEFAULT_RECONNECT_DELAY_SECS: u64 = 5;
pub const RETRY_TTL_MS: u32 = 15000; // 15 seconds before retry
pub const MAX_RETRIES: u32 = 3;

/// Returns RabbitMQ URI from env RBMQ_URI or default.
pub fn rabbitmq_uri() -> String {
    std::env::var("RBMQ_URI").unwrap_or_else(|_| "amqp://adminUser:strongPassword@localhost:5672/%2f".to_string())
}

pub fn reconnect_delay() -> Duration {
    Duration::from_secs(DEFAULT_RECONNECT_DELAY_SECS)
}
