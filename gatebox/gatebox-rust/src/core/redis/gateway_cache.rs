use crate::core::redis::errors::RedisError;
use crate::core::redis::Client;
use crate::core::redis::ttl::GATEWAY_CONFIG_TTL;
use tracing::warn;

pub struct GatewayCache {
    client: Client,
}

impl GatewayCache {
    pub fn new(client: Client) -> Self {
        GatewayCache { client }
    }

    pub async fn get_gateway_config(&self) -> Result<Vec<u8>, RedisError> {
        let key = "gateway:config:all";
        self.client.get(key).await.map_err(|e| {
            if !matches!(e, RedisError::KeyNotFound) {
                warn!("Erro ao buscar GatewayConfig do cache: {}", e);
            }
            e
        }).map(|s| s.into_bytes())
    }

    pub async fn set_gateway_config(&self, json: &[u8]) -> Result<(), RedisError> {
        let key = "gateway:config:all";
        let s = String::from_utf8_lossy(json);
        self.client.set(key, s.as_ref(), GATEWAY_CONFIG_TTL).await.map_err(|e| {
            warn!("Erro ao armazenar GatewayConfig no cache: {}", e);
            e
        })
    }

    pub async fn invalidate_gateway_config(&self) -> Result<(), RedisError> {
        let key = "gateway:config:all";
        if let Err(e) = self.client.delete(key).await {
            if !matches!(e, RedisError::Unavailable) {
                warn!("Erro ao invalidar GatewayConfig: {}", e);
                return Err(e);
            }
        }
        Ok(())
    }

    fn item_key(gateway_name: &str) -> String {
        format!("gateway:config:{}", gateway_name)
    }

    pub async fn get_gateway_config_item(&self, gateway_name: &str) -> Result<Vec<u8>, RedisError> {
        let key = Self::item_key(gateway_name);
        self.client.get(&key).await.map_err(|e| {
            if !matches!(e, RedisError::KeyNotFound) {
                warn!("Erro ao buscar GatewayConfigItem do cache (gateway={}): {}", gateway_name, e);
            }
            e
        }).map(|s| s.into_bytes())
    }

    pub async fn set_gateway_config_item(&self, gateway_name: &str, json: &[u8]) -> Result<(), RedisError> {
        let key = Self::item_key(gateway_name);
        let s = String::from_utf8_lossy(json);
        self.client.set(&key, s.as_ref(), GATEWAY_CONFIG_TTL).await.map_err(|e| {
            warn!("Erro ao armazenar GatewayConfigItem no cache (gateway={}): {}", gateway_name, e);
            e
        })
    }
}
