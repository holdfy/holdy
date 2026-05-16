//! `GET /admin/reports/yield`

use axum::{extract::Query, extract::State, Json};
use rust_decimal::Decimal;

use crate::dto::{YieldReportQuery, YieldReportResponse};
use crate::error::AdminError;
use crate::state::AdminState;
use apicash_custody::CustodyStatus;

pub async fn get_yield_report(
    State(state): State<AdminState>,
    Query(q): Query<YieldReportQuery>,
) -> Result<Json<YieldReportResponse>, AdminError> {
    let custodies = state.custody.list_all_custodies().await?;

    let from = q.from;
    let to = q.to;

    let filtered: Vec<_> = custodies
        .into_iter()
        .filter(|c| {
            let t = c.locked_at;
            if let Some(f) = from {
                if t < f {
                    return false;
                }
            }
            if let Some(until) = to {
                if t > until {
                    return false;
                }
            }
            true
        })
        .collect();

    let total_yield: Decimal = filtered
        .iter()
        .filter_map(|c| c.yield_earned.as_ref())
        .map(|y| y.decimal())
        .sum();

    let released_count = filtered
        .iter()
        .filter(|c| c.status == CustodyStatus::Released)
        .count();

    Ok(Json(YieldReportResponse {
        total_yield_minor: total_yield.to_string(),
        custody_count: filtered.len(),
        released_count,
        period_from: from,
        period_to: to,
    }))
}
