//! `GET /admin/orders`

use axum::{extract::Query, extract::State, Json};

use crate::dto::{AdminOrderListResponse, AdminOrderRow, OrderListQuery};
use crate::error::AdminError;
use crate::state::AdminState;

pub async fn list_orders(
    State(state): State<AdminState>,
    Query(q): Query<OrderListQuery>,
) -> Result<Json<AdminOrderListResponse>, AdminError> {
    let orders = state
        .orders
        .list_all()
        .await
        .map_err(AdminError::internal)?;
    let mut rows: Vec<AdminOrderRow> = Vec::new();

    for s in orders {
        if let Some(st) = q.status {
            if s.order.status != st {
                continue;
            }
        }
        if let Some(min) = q.min_score {
            if s.risk_score < min {
                continue;
            }
        }
        if let Some(from) = q.from {
            if s.order.created_at < from {
                continue;
            }
        }
        if let Some(to) = q.to {
            if s.order.created_at > to {
                continue;
            }
        }

        rows.push(AdminOrderRow {
            order_id: s.order.id,
            buyer_id: s.order.buyer_id,
            seller_id: s.order.seller_id,
            amount_minor: s.order.amount.decimal().to_string(),
            status: s.order.status,
            risk_score: s.risk_score,
            risk_decision: s.risk_decision.clone(),
            custody_id: s.custody_id,
            created_at: s.order.created_at,
            platform_origin: s.platform_origin,
        });
    }

    let total = rows.len();
    Ok(Json(AdminOrderListResponse {
        orders: rows,
        total,
    }))
}
