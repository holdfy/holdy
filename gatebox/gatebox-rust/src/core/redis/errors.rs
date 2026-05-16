// From app/modules/core/redis/errors.go
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("redis não disponível, usando fallback para PostgreSQL")]
    Unavailable,

    #[error("chave não encontrada no cache")]
    KeyNotFound,

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

pub use RedisError::{KeyNotFound, Unavailable as ErrRedisUnavailable};
