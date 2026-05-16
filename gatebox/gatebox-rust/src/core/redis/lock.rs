use std::time::Duration;

use crate::core::redis::errors::{RedisError, ErrRedisUnavailable};
use crate::core::redis::Client;
use crate::core::redis::ttl::LOCK_TTL;
use tracing::warn;

pub struct Lock {
    client: Client,
}

impl Lock {
    pub fn new(client: Client) -> Self {
        Lock { client }
    }

    fn key(resource: &str, resource_id: i64) -> String {
        format!("lock:{}:{}", resource, resource_id)
    }

    pub async fn acquire(&self, resource: &str, resource_id: i64) -> Result<bool, RedisError> {
        let key = Self::key(resource, resource_id);
        match self.client.set_nx(&key, "locked", LOCK_TTL).await {
            Ok(acquired) => Ok(acquired),
            Err(ErrRedisUnavailable) => {
                warn!(
                    "Redis indisponível para lock (resource={}, id={}), permitindo operação",
                    resource, resource_id
                );
                Ok(true)
            }
            Err(e) => {
                warn!("Erro ao adquirir lock (resource={}, id={}): {}", resource, resource_id, e);
                Err(e)
            }
        }
    }

    pub async fn release(&self, resource: &str, resource_id: i64) -> Result<(), RedisError> {
        let key = Self::key(resource, resource_id);
        if let Err(e) = self.client.delete(&key).await {
            if !matches!(e, RedisError::Unavailable) {
                warn!("Erro ao liberar lock (resource={}, id={}): {}", resource, resource_id, e);
                return Err(e);
            }
        }
        Ok(())
    }

    pub async fn acquire_with_retry(
        &self,
        resource: &str,
        resource_id: i64,
        max_retries: u32,
        retry_interval: Duration,
    ) -> Result<bool, RedisError> {
        for i in 0..max_retries {
            match self.acquire(resource, resource_id).await {
                Ok(true) => return Ok(true),
                Ok(false) => {}
                Err(e) if matches!(e, RedisError::Unavailable) => {}
                Err(e) => return Err(e),
            }
            if i < max_retries - 1 {
                tokio::time::sleep(retry_interval).await;
            }
        }
        Err(RedisError::Other(anyhow::anyhow!(
            "não foi possível adquirir lock após {} tentativas",
            max_retries
        )))
    }
}
