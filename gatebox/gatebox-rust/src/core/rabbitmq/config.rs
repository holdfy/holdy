// From app/modules/core/rabbitmq/config.go (constants and config struct)
use std::time::Duration;

pub const QUEUE_NAME_PAYMENT: &str = "payment-queue";
pub const QUEUE_NAME_DLX: &str = "payment-queue-dlx";
pub const QUEUE_NAME_HOSPITAL: &str = "payment-queue-hospital";
pub const EXCHANGE_NAME_DLX: &str = "payment-exchange-dlx";
pub const EXCHANGE_NAME_HOSPITAL: &str = "payment-exchange-hospital";
pub const DEFAULT_RECONNECT_DELAY_SECS: u64 = 5;
pub const RETRY_TTL_MS: u32 = 15000;
pub const MAX_RETRIES: u32 = 3;

/// RabbitMQ connection/config. POC: env-based URI only.
#[derive(Debug, Clone)]
pub struct RabbitMQConfig {
    pub uri: String,
    pub queue_name: String,
    pub reconnect_delay: Duration,
}

impl Default for RabbitMQConfig {
    fn default() -> Self {
        let uri = std::env::var("RBMQ_URI")
            .unwrap_or_else(|_| "amqp://adminUser:strongPassword@localhost:5672/%2f".to_string());
        RabbitMQConfig {
            uri,
            queue_name: QUEUE_NAME_PAYMENT.to_string(),
            reconnect_delay: Duration::from_secs(DEFAULT_RECONNECT_DELAY_SECS),
        }
    }
}
