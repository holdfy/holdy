use crate::core::redis::errors::RedisError;
use crate::core::redis::Client;
use crate::core::redis::ttl::PROVIDER_INFO_TTL;
use tracing::warn;

pub struct ProviderCache {
    client: Client,
}

impl ProviderCache {
    pub fn new(client: Client) -> Self {
        ProviderCache { client }
    }

    fn key(account_id: i64, gateway_name: &str) -> String {
        format!("provider:{}:{}", account_id, gateway_name)
    }

    pub async fn get_provider_info(&self, account_id: i64, gateway_name: &str) -> Result<Vec<u8>, RedisError> {
        let key = Self::key(account_id, gateway_name);
        let s = self.client.get(&key).await.map_err(|e| {
            if !matches!(e, RedisError::KeyNotFound) {
                warn!(
                    "Erro ao buscar ProviderInfo do cache (account={}, gateway={}): {}",
                    account_id, gateway_name, e
                );
            }
            e
        })?;
        Ok(s.into_bytes())
    }

    pub async fn set_provider_info(&self, account_id: i64, gateway_name: &str, json: &[u8]) -> Result<(), RedisError> {
        let key = Self::key(account_id, gateway_name);
        let s = String::from_utf8_lossy(json);
        self.client.set(&key, s.as_ref(), PROVIDER_INFO_TTL).await.map_err(|e| {
            warn!(
                "Erro ao armazenar ProviderInfo no cache (account={}, gateway={}): {}",
                account_id, gateway_name, e
            );
            e
        })
    }
}
