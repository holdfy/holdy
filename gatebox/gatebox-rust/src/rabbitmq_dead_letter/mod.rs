// RabbitMQ dead letter POC (from app/modules/core/rabbitmq: config, hospital consumer).
pub mod config;
pub mod hospital_consumer;
pub mod types;

pub use config::{
    rabbitmq_uri, reconnect_delay, QUEUE_NAME_PAYMENT, QUEUE_NAME_DLX, QUEUE_NAME_HOSPITAL,
    EXCHANGE_NAME_DLX, EXCHANGE_NAME_HOSPITAL, RETRY_TTL_MS, MAX_RETRIES,
};
pub use hospital_consumer::{handle_hospital_delivery, run_hospital_consumer};
pub use types::PaymentMessage;
