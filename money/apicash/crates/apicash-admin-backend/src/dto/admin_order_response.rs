//! Lista de pedidos com metadados administrativos.

use apicash_shared::OrderStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct OrderListQuery {
    pub status: Option<OrderStatus>,
    pub min_score: Option<u32>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct AdminOrderRow {
    pub order_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub amount_minor: String,
    pub status: OrderStatus,
    pub risk_score: u32,
    pub risk_decision: String,
    pub custody_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminOrderListResponse {
    pub orders: Vec<AdminOrderRow>,
    pub total: usize,
}
