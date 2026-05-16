// TTLs from app/modules/core/redis (provider_cache, pix_key_cache, etc.)
use std::time::Duration;

pub const PIX_KEY_TTL: Duration = Duration::from_secs(30 * 60);       // 30 min
pub const PROVIDER_INFO_TTL: Duration = Duration::from_secs(10 * 60); // 10 min
pub const GATEWAY_CONFIG_TTL: Duration = Duration::from_secs(5 * 60); // 5 min
pub const GATEWAY_TOKEN_TTL: Duration = Duration::from_secs(59 * 60); // 59 min
pub const AUTH_ID_TTL: Duration = Duration::from_secs(5 * 60);       // 5 min
pub const LOCK_TTL: Duration = Duration::from_secs(30);              // 30 s
pub const BALANCE_TTL: Duration = Duration::from_secs(30);            // 30 s
