// From app/modules/core/rabbitmq - config, types, producer/consumer, pools
pub mod config;
pub mod consumer;
pub mod producer;
pub mod producer_pool;
pub mod types;
pub mod worker_pool;

pub use config::{RabbitMQConfig, QUEUE_NAME_PAYMENT, QUEUE_NAME_DLX, QUEUE_NAME_HOSPITAL};
pub use producer_pool::{ProducerPool, ProducerPoolConfig, PublishMessage};
pub use types::{GatewayFailureConfig, MessageHandler, PaymentMessage};
pub use worker_pool::{WorkerPool, WorkerPoolConfig};
