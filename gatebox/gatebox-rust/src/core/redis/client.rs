// From app/modules/core/redis/client.go
use crate::core::redis::errors::{RedisError, ErrRedisUnavailable, KeyNotFound};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::warn;

/// Redis client with optional disabled state (fallback to PostgreSQL when Redis is off).
pub struct Client {
    conn: Option<Arc<Mutex<redis::aio::MultiplexedConnection>>>,
    enabled: bool,
}

impl Client {
    /// Creates a new Redis client. If REDIS_ENABLED=false or ping fails, returns a disabled client.
    pub async fn new() -> Result<Self, RedisError> {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let enabled_env = std::env::var("REDIS_ENABLED").unwrap_or_else(|_| "true".to_string());

        if enabled_env == "false" {
            tracing::info!("Redis desabilitado via REDIS_ENABLED=false, usando apenas PostgreSQL");
            return Ok(Client {
                conn: None,
                enabled: false,
            });
        }

        let client =
            redis::Client::open(redis_url.as_str()).map_err(|e| anyhow::anyhow!("{}", e))?;
        let mut conn = client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        if redis::cmd("PING").query_async::<_, String>(&mut conn).await.is_err() {
            warn!("Redis não disponível, usando fallback para PostgreSQL");
            return Ok(Client {
                conn: None,
                enabled: false,
            });
        }

        tracing::info!("Redis conectado com sucesso em {}", redis_url);
        Ok(Client {
            conn: Some(Arc::new(Mutex::new(conn))),
            enabled: true,
        })
    }

    pub async fn get(&self, key: &str) -> Result<String, RedisError> {
        if !self.enabled {
            return Err(ErrRedisUnavailable);
        }
        let conn = self.conn.as_ref().ok_or(ErrRedisUnavailable)?;
        let mut conn = conn.lock().await;
        let v: Result<Option<String>, _> = redis::cmd("GET").arg(key).query_async(&mut *conn).await;
        match v {
            Err(e) => {
                warn!("Erro ao buscar chave Redis {}: {}", key, e);
                Err(anyhow::anyhow!("{}", e).into())
            }
            Ok(Some(s)) => Ok(s),
            Ok(None) => Err(KeyNotFound),
        }
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, RedisError> {
        let s = self.get(key).await?;
        serde_json::from_str(&s).map_err(|e| anyhow::anyhow!("erro ao deserializar JSON do Redis: {}", e).into())
    }

    pub async fn set<V: redis::ToRedisArgs + Send + Sync>(
        &self,
        key: &str,
        value: V,
        ttl: Duration,
    ) -> Result<(), RedisError> {
        if !self.enabled {
            return Err(ErrRedisUnavailable);
        }
        let conn = self.conn.as_ref().ok_or(ErrRedisUnavailable)?;
        let mut conn = conn.lock().await;
        let secs = ttl.as_secs() as i64;
        let _: () = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("EX")
            .arg(secs)
            .query_async(&mut *conn)
            .await
            .map_err(|e: redis::RedisError| -> RedisError {
                warn!("Erro ao armazenar chave Redis {}: {}", key, e);
                anyhow::anyhow!("{}", e).into()
            })?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), RedisError> {
        if !self.enabled {
            return Err(ErrRedisUnavailable);
        }
        let conn = self.conn.as_ref().ok_or(ErrRedisUnavailable)?;
        let mut conn = conn.lock().await;
        let _: () = redis::cmd("DEL")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .map_err(|e: redis::RedisError| -> RedisError {
                warn!("Erro ao deletar chave Redis {}: {}", key, e);
                anyhow::anyhow!("{}", e).into()
            })?;
        Ok(())
    }

    pub async fn set_nx<V: redis::ToRedisArgs + Send + Sync>(
        &self,
        key: &str,
        value: V,
        ttl: Duration,
    ) -> Result<bool, RedisError> {
        if !self.enabled {
            return Err(ErrRedisUnavailable);
        }
        let conn = self.conn.as_ref().ok_or(ErrRedisUnavailable)?;
        let mut conn = conn.lock().await;
        let secs = ttl.as_secs() as i64;
        let ok: bool = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("EX")
            .arg(secs)
            .arg("NX")
            .query_async(&mut *conn)
            .await
            .map_err(|e: redis::RedisError| -> RedisError {
                warn!("Erro ao executar SetNX na chave Redis {}: {}", key, e);
                anyhow::anyhow!("{}", e).into()
            })?;
        Ok(ok)
    }

    pub async fn exists(&self, key: &str) -> Result<bool, RedisError> {
        if !self.enabled {
            return Err(ErrRedisUnavailable);
        }
        let conn = self.conn.as_ref().ok_or(ErrRedisUnavailable)?;
        let mut conn = conn.lock().await;
        let n: i32 = redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .map_err(|e: redis::RedisError| -> RedisError {
                warn!("Erro ao verificar existência da chave Redis {}: {}", key, e);
                anyhow::anyhow!("{}", e).into()
            })?;
        Ok(n > 0)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled && self.conn.is_some()
    }
}
