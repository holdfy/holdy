//! `GET /admin/dashboard`

use axum::{extract::State, Json};
use rust_decimal::Decimal;

use crate::dto::AdminDashboardResponse;
use crate::error::AdminError;
use crate::state::AdminState;
use apicash_custody::CustodyStatus;
use apicash_disputes::DisputeStatus;

pub async fn get_dashboard(
    State(state): State<AdminState>,
) -> Result<Json<AdminDashboardResponse>, AdminError> {
    let custodies = state.custody.list_all_custodies().await?;
    let disputes = state.disputes.list_all_disputes().await?;

    let total_volume: Decimal = custodies.iter().map(|c| c.amount.decimal()).sum();
    let total_yield: Decimal = custodies
        .iter()
        .filter_map(|c| c.yield_earned.as_ref())
        .map(|y| y.decimal())
        .sum();

    let open_disputes = disputes
        .iter()
        .filter(|d| matches!(d.status, DisputeStatus::Open | DisputeStatus::UnderReview))
        .count();

    let locked_custodies = custodies
        .iter()
        .filter(|c| matches!(c.status, CustodyStatus::Locked | CustodyStatus::Disputed))
        .count();

    Ok(Json(AdminDashboardResponse {
        total_volume_minor: total_volume.to_string(),
        total_yield_accrued_minor: total_yield.to_string(),
        open_disputes,
        locked_custodies,
    }))
}
