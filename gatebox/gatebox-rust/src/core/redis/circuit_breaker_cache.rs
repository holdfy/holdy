// From app/modules/core/redis/circuit_breaker_cache.go
use crate::core::redis::errors::RedisError;
use crate::core::redis::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::warn;

/// TTL for Circuit Breaker state cache (from provider_cache.go).
pub const CIRCUIT_BREAKER_TTL: Duration = Duration::from_secs(60);

/// Circuit Breaker state stored in Redis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub state: String,             // CLOSED, OPEN, HALF_OPEN
    pub consecutive_errors: i32,
    pub consecutive_success: i32,
    pub last_error_time: Option<String>, // ISO8601
    pub opened_at: Option<String>,
}

/// Cache for Circuit Breaker state (Redis).
pub struct CircuitBreakerCache {
    client: Client,
}

impl CircuitBreakerCache {
    pub fn new(client: Client) -> Self {
        CircuitBreakerCache { client }
    }

    fn key(gateway_name: &str) -> String {
        format!("cb:state:{}", gateway_name)
    }

    pub async fn get_circuit_breaker_state(
        &self,
        gateway_name: &str,
    ) -> Result<CircuitBreakerState, RedisError> {
        let key = Self::key(gateway_name);
        self.client
            .get_json(&key)
            .await
            .map_err(|e| {
                if matches!(e, RedisError::KeyNotFound) {
                    return e;
                }
                warn!(
                    "Erro ao buscar CircuitBreakerState do cache (gateway={}): {}",
                    gateway_name, e
                );
                e
            })
    }

    pub async fn set_circuit_breaker_state(
        &self,
        gateway_name: &str,
        state: &CircuitBreakerState,
    ) -> Result<(), RedisError> {
        let key = Self::key(gateway_name);
        let json = serde_json::to_string(state).map_err(|e| anyhow::anyhow!("{}", e))?;
        self.client
            .set(&key, json, CIRCUIT_BREAKER_TTL)
            .await
            .map_err(|e| {
                warn!(
                    "Erro ao armazenar CircuitBreakerState no cache (gateway={}): {}",
                    gateway_name, e
                );
                e
            })
    }

    pub async fn invalidate_circuit_breaker_state(&self, gateway_name: &str) -> Result<(), RedisError> {
        let key = Self::key(gateway_name);
        if let Err(e) = self.client.delete(&key).await {
            if !matches!(e, RedisError::Unavailable) {
                warn!(
                    "Erro ao invalidar CircuitBreakerState (gateway={}): {}",
                    gateway_name, e
                );
                return Err(e);
            }
        }
        Ok(())
    }
}
