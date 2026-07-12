//! APICash shared library: configuration, errors, money types, and domain models.
//!
//! All monetary values use [`Money`](types::Money) ([`rust_decimal::Decimal`] internally), never `f64`.

pub mod audit;
pub mod config;
pub mod constants;
pub mod error;
pub mod logging;
pub mod models;
pub mod prelude;
pub mod types;
pub mod utils;

// --- Config & errors ---
pub use audit::AuditEvent;
pub use config::{
    AntiFraudConfig, AppConfig, AuthConfig, DatabaseConfig, NotificationsConfig, PulsarConfig,
    RedisConfig, StellarConfig, WhatsAppConfig,
};
pub use error::ApiCashError;

// --- Types ---
pub use types::{CustodyStatus, DisputeStatus, Money, OrderStatus, PaymentStatus, PlatformOrigin};

// --- Models ---
pub use models::{Custody, Dispute, Order, Payment, RiskLevel, ScoreFactor, User, UserScore};

// --- Constants ---
pub use constants::{
    DEFAULT_CUSTODY_DAYS, DEFAULT_MAX_BODY_BYTES, MAX_MONEY_STR_LEN, PULSAR_NAMESPACE_DEFAULT,
    USER_SCORE_MAX, USER_SCORE_MIN, YIELD_DISTRIBUTION_PERCENT,
};

// --- Utils ---
pub use utils::{
    assert_testnet_live_config, assert_x402_config, default_horizon_url, default_soroban_rpc_url,
    facilitator_url, minio_object_url, network_label, network_passphrase, parse_network_label,
    pay_to_address, price_usdc, public_base_url, require_testnet, require_x402,
    validate_testnet_live_config,
    validate_x402_config, StellarNetworkKind,
};
