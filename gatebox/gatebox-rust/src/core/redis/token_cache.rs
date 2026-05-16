use std::time::Duration;

use crate::core::redis::errors::RedisError;
use crate::core::redis::Client;
use crate::core::redis::ttl::GATEWAY_TOKEN_TTL;
use tracing::warn;

pub struct TokenCache {
    client: Client,
}

impl TokenCache {
    pub fn new(client: Client) -> Self {
        TokenCache { client }
    }

    fn key(gateway_name: &str, client_id: &str) -> String {
        format!("gateway:token:{}:{}", gateway_name, client_id)
    }

    pub async fn get_gateway_token(&self, gateway_name: &str, client_id: &str) -> Result<String, RedisError> {
        let key = Self::key(gateway_name, client_id);
        self.client.get(&key).await.map_err(|e| {
            if !matches!(e, RedisError::KeyNotFound) {
                warn!(
                    "Erro ao buscar token do cache (gateway={}, clientID={}): {}",
                    gateway_name, client_id, e
                );
            }
            e
        })
    }

    pub async fn set_gateway_token(
        &self,
        gateway_name: &str,
        client_id: &str,
        token: &str,
        ttl: Option<Duration>,
    ) -> Result<(), RedisError> {
        let key = Self::key(gateway_name, client_id);
        let ttl = ttl.unwrap_or(GATEWAY_TOKEN_TTL);
        self.client.set(&key, token, ttl).await.map_err(|e| {
            warn!(
                "Erro ao armazenar token no cache (gateway={}, clientID={}): {}",
                gateway_name, client_id, e
            );
            e
        })
    }

    pub async fn invalidate_gateway_token(&self, gateway_name: &str, client_id: &str) -> Result<(), RedisError> {
        let key = Self::key(gateway_name, client_id);
        if let Err(e) = self.client.delete(&key).await {
            if !matches!(e, RedisError::Unavailable) {
                warn!(
                    "Erro ao invalidar token (gateway={}, clientID={}): {}",
                    gateway_name, client_id, e
                );
                return Err(e);
            }
        }
        Ok(())
    }
}
