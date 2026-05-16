//! `GET /admin/sellers/:id/dashboard`

use std::collections::HashSet;

use axum::{extract::Path, extract::State, Json};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::dto::SellerDashboardResponse;
use crate::error::AdminError;
use crate::state::AdminState;
use apicash_disputes::DisputeStatus;

pub async fn get_seller_dashboard(
    State(state): State<AdminState>,
    Path(seller_id): Path<Uuid>,
) -> Result<Json<SellerDashboardResponse>, AdminError> {
    let (order_count, volume, score_sum, score_n, seller_order_ids) = {
        let orders = state
            .orders
            .list_all()
            .await
            .map_err(AdminError::internal)?;
        let mut order_count = 0u64;
        let mut volume = Decimal::ZERO;
        let mut score_sum = 0u64;
        let mut score_n = 0u64;
        let mut ids = HashSet::new();
        for o in orders {
            if o.order.seller_id != seller_id {
                continue;
            }
            order_count += 1;
            volume += o.order.amount.decimal();
            score_sum += u64::from(o.risk_score);
            score_n += 1;
            ids.insert(o.order.id);
        }
        (order_count, volume, score_sum, score_n, ids)
    };

    let disputes = state.disputes.list_all_disputes().await?;
    let open_disputes = disputes
        .into_iter()
        .filter(|d| matches!(d.status, DisputeStatus::Open | DisputeStatus::UnderReview))
        .filter(|d| seller_order_ids.contains(&d.order_id))
        .count() as u64;

    let average_risk_score = if score_n > 0 {
        (Decimal::from(score_sum) / Decimal::from(score_n))
            .normalize()
            .to_string()
    } else {
        Decimal::ZERO.to_string()
    };

    Ok(Json(SellerDashboardResponse {
        seller_id,
        order_count,
        total_volume_minor: volume.to_string(),
        average_risk_score,
        open_disputes,
    }))
}
