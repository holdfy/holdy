use crate::core::redis::errors::RedisError;
use crate::core::redis::Client;
use crate::core::redis::ttl::AUTH_ID_TTL;
use tracing::warn;

pub struct AuthCache {
    client: Client,
}

impl AuthCache {
    pub fn new(client: Client) -> Self {
        AuthCache { client }
    }

    fn key(account_id: i64) -> String {
        format!("account:auth_id:{}", account_id)
    }

    pub async fn get_authentication_id(&self, account_id: i64) -> Result<i64, RedisError> {
        let key = Self::key(account_id);
        let s = self.client.get(&key).await.map_err(|e| {
            if !matches!(e, RedisError::KeyNotFound) {
                warn!("Erro ao buscar authentication_id do cache (account={}): {}", account_id, e);
            }
            e
        })?;
        s.parse::<i64>().map_err(|_| {
            warn!("Erro ao converter authentication_id do cache (account={}, val={})", account_id, s);
            RedisError::Other(anyhow::anyhow!("erro ao converter authentication_id"))
        })
    }

    pub async fn set_authentication_id(&self, account_id: i64, auth_id: i64) -> Result<(), RedisError> {
        let key = Self::key(account_id);
        let value = auth_id.to_string();
        self.client.set(&key, value.as_str(), AUTH_ID_TTL).await.map_err(|e| {
            warn!(
                "Erro ao armazenar authentication_id no cache (account={}, authID={}): {}",
                account_id, auth_id, e
            );
            e
        })
    }

    pub async fn invalidate_authentication_id(&self, account_id: i64) -> Result<(), RedisError> {
        let key = Self::key(account_id);
        if let Err(e) = self.client.delete(&key).await {
            if !matches!(e, RedisError::Unavailable) {
                warn!("Erro ao invalidar authentication_id (account={}): {}", account_id, e);
                return Err(e);
            }
        }
        Ok(())
    }
}
