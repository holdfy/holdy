//! Resumo por vendedor.

use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SellerDashboardResponse {
    pub seller_id: Uuid,
    pub order_count: u64,
    pub total_volume_minor: String,
    /// Média do score 0–1000 como string decimal (ex.: `"480.5"`).
    pub average_risk_score: String,
    pub open_disputes: u64,
}
