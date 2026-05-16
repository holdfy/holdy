//! Resumo do painel.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AdminDashboardResponse {
    pub total_volume_minor: String,
    pub total_yield_accrued_minor: String,
    pub open_disputes: usize,
    pub locked_custodies: usize,
}
