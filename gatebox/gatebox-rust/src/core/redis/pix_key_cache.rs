use crate::core::redis::errors::RedisError;
use crate::core::redis::Client;
use crate::core::redis::ttl::PIX_KEY_TTL;
use tracing::warn;

pub struct PixKeyCache {
    client: Client,
}

impl PixKeyCache {
    pub fn new(client: Client) -> Self {
        PixKeyCache { client }
    }

    fn key(account_id: i64) -> String {
        format!("pix_key:account:{}", account_id)
    }

    pub async fn get_pix_key(&self, account_id: i64) -> Result<String, RedisError> {
        let key = Self::key(account_id);
        self.client.get(&key).await.map_err(|e| {
            if !matches!(e, RedisError::KeyNotFound) {
                warn!("Erro ao buscar PixKey do cache (account={}): {}", account_id, e);
            }
            e
        })
    }

    pub async fn set_pix_key(&self, account_id: i64, pix_key: &str) -> Result<(), RedisError> {
        let key = Self::key(account_id);
        self.client.set(&key, pix_key, PIX_KEY_TTL).await.map_err(|e| {
            warn!("Erro ao armazenar PixKey no cache (account={}): {}", account_id, e);
            e
        })
    }

    pub async fn invalidate_pix_key(&self, account_id: i64) -> Result<(), RedisError> {
        let key = Self::key(account_id);
        if let Err(e) = self.client.delete(&key).await {
            if !matches!(e, RedisError::Unavailable) {
                warn!("Erro ao invalidar PixKey: {}", e);
                return Err(e);
            }
        }
        Ok(())
    }
}
