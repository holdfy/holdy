// From app/modules/core/rabbitmq/worker_pool.go - runs real Consumer
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;

use super::config::RabbitMQConfig;
use super::consumer::Consumer;
use super::types::MessageHandler;

#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    pub num_workers: usize,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub shutdown_timeout: Duration,
    pub enable_metrics: bool,
    pub dedicated_connection_per_worker: bool,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        WorkerPoolConfig {
            num_workers: 10,
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
            shutdown_timeout: Duration::from_secs(30),
            enable_metrics: true,
            dedicated_connection_per_worker: true,
        }
    }
}

/// Worker pool - runs one Consumer in a spawned task; stop() signals cancel.
pub struct WorkerPool {
    config: RabbitMQConfig,
    _pool_config: WorkerPoolConfig,
    handler: Arc<dyn MessageHandler>,
    cancel_tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<()>>>>,
    join_handle: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>>>>,
}

impl WorkerPool {
    pub fn new(pool_config: WorkerPoolConfig, handler: Arc<dyn MessageHandler>) -> Self {
        WorkerPool {
            config: RabbitMQConfig::default(),
            _pool_config: pool_config,
            handler,
            cancel_tx: Arc::new(tokio::sync::Mutex::new(None)),
            join_handle: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    pub fn with_config(mut self, config: RabbitMQConfig) -> Self {
        self.config = config;
        self
    }

    /// Start the pool: spawns Consumer::run in a background task.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let consumer = Consumer::new(self.config.clone(), Arc::clone(&self.handler));
        let join = tokio::spawn(async move { consumer.run(rx).await });
        *self.cancel_tx.lock().await = Some(tx);
        *self.join_handle.lock().await = Some(join);
        Ok(())
    }

    /// Stop the pool: signal cancel and wait for the consumer task.
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx_guard = self.cancel_tx.lock().await;
        if let Some(tx) = tx_guard.take() {
            let _ = tx.send(());
        }
        drop(tx_guard);
        let mut join_guard = self.join_handle.lock().await;
        if let Some(join) = join_guard.take() {
            let _ = join.await;
        }
        Ok(())
    }
}
