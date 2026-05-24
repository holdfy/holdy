//! Behavioral signals assembled by the service before scoring.

use rust_decimal::Decimal;

/// Runtime context derived from transaction history and dispute records.
/// Built by [`AntiFraudeService`] from repository queries; passed to [`ScoreCalculator`].
#[derive(Debug, Clone)]
pub struct BehavioralContext {
    /// Disputes currently open that were opened BY this user.
    pub open_dispute_count: u32,
    /// Disputes currently open that were opened AGAINST this user by a counterparty.
    pub disputes_as_counterparty: u32,
    /// `disputes_total / tx_count_total` — 0.0 when no history.
    pub dispute_rate: f64,
    /// Number of orders created in the last 24 hours.
    pub tx_count_24h: u32,
    /// Total orders ever created by this user on the platform.
    pub tx_count_total: u32,
    /// Sum of order amounts (BRL) in the last 24 hours.
    pub tx_volume_24h_brl: Decimal,
    /// Historical average order value for this user; `None` if no prior orders.
    pub avg_tx_value: Option<Decimal>,
    /// Age of the user's account in days (proxy: days since first order).
    pub account_age_days: u32,
    /// Amount of the current transaction being evaluated (for structuring + anomaly checks).
    pub current_tx_amount: Option<Decimal>,
}

impl BehavioralContext {
    /// Neutral context used in tests and flows where behavioral data is unavailable.
    pub fn neutral() -> Self {
        Self {
            open_dispute_count: 0,
            disputes_as_counterparty: 0,
            dispute_rate: 0.0,
            tx_count_24h: 0,
            tx_count_total: 0,
            tx_volume_24h_brl: Decimal::ZERO,
            avg_tx_value: None,
            account_age_days: 0,
            current_tx_amount: None,
        }
    }
}
