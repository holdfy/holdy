//! Global constants for APICash business rules and defaults.

/// Default number of days funds stay in custody before auto-release policies apply.
pub const DEFAULT_CUSTODY_DAYS: u32 = 7;

/// Yield distribution split as percentages: `(platform_bps, liquidity_bps, reserve_bps)` — must sum to 100.
/// Example: 70% platform share of distributable yield, 10% liquidity incentive, 20% reserve.
pub const YIELD_DISTRIBUTION_PERCENT: (u8, u8, u8) = (70, 10, 20);

/// Maximum `Money` amount string length accepted by parsers (safety bound).
pub const MAX_MONEY_STR_LEN: usize = 64;

/// Default API request body limit hint (bytes); services may override.
pub const DEFAULT_MAX_BODY_BYTES: usize = 1024 * 1024;

/// Pulsar default tenant namespace separator (convention).
pub const PULSAR_NAMESPACE_DEFAULT: &str = "default";

/// Minimum user score (inclusive).
pub const USER_SCORE_MIN: u32 = 0;

/// Maximum user score (inclusive).
pub const USER_SCORE_MAX: u32 = 1000;
