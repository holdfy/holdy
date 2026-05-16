use crate::core::redis::errors::RedisError;
use crate::core::redis::Client;
use crate::core::redis::ttl::BALANCE_TTL;
use tracing::warn;

pub struct BalanceCache {
    client: Client,
}

impl BalanceCache {
    pub fn new(client: Client) -> Self {
        BalanceCache { client }
    }

    fn key(account_id: i64) -> String {
        format!("balance:{}", account_id)
    }

    pub async fn get_balance(&self, account_id: i64) -> Result<f64, RedisError> {
        let key = Self::key(account_id);
        let s = self.client.get(&key).await.map_err(|e| {
            if !matches!(e, RedisError::KeyNotFound) {
                warn!("Erro ao buscar saldo do cache (account={}): {}", account_id, e);
            }
            e
        })?;
        s.parse::<f64>().map_err(|_| {
            warn!("Erro ao parsear saldo do cache (account={}, value={})", account_id, s);
            RedisError::Other(anyhow::anyhow!("invalid balance value in cache"))
        })
    }

    pub async fn set_balance(&self, account_id: i64, balance: f64) -> Result<(), RedisError> {
        let key = Self::key(account_id);
        let value = format!("{:.2}", balance);
        self.client.set(&key, value.as_str(), BALANCE_TTL).await.map_err(|e| {
            warn!("Erro ao cachear saldo (account={}): {}", account_id, e);
            e
        })
    }

    pub async fn invalidate_balance(&self, account_id: i64) -> Result<(), RedisError> {
        let key = Self::key(account_id);
        if let Err(e) = self.client.delete(&key).await {
            if !matches!(e, RedisError::Unavailable) {
                warn!("Erro ao invalidar saldo (account={}): {}", account_id, e);
                return Err(e);
            }
        }
        Ok(())
    }
}
