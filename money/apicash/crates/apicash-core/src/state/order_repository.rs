//! Persistence boundary for orders.

use std::collections::HashMap;
use std::sync::Arc;

use apicash_shared::{Money, Order, OrderStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::app_state::StoredOrder;

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn save(&self, order: StoredOrder) -> Result<(), String>;
    async fn get(&self, id: Uuid) -> Result<Option<StoredOrder>, String>;
    async fn update(&self, order: StoredOrder) -> Result<(), String>;
    async fn list_all(&self) -> Result<Vec<StoredOrder>, String>;
    async fn find_by_gateway_tx_id(&self, tx_id: &str) -> Result<Option<StoredOrder>, String>;
}

pub struct InMemoryOrderRepository {
    by_id: Arc<RwLock<HashMap<Uuid, StoredOrder>>>,
}

impl InMemoryOrderRepository {
    pub fn new() -> Self {
        Self {
            by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryOrderRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OrderRepository for InMemoryOrderRepository {
    async fn save(&self, order: StoredOrder) -> Result<(), String> {
        self.by_id.write().await.insert(order.order.id, order);
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<StoredOrder>, String> {
        Ok(self.by_id.read().await.get(&id).cloned())
    }

    async fn update(&self, order: StoredOrder) -> Result<(), String> {
        let mut g = self.by_id.write().await;
        if !g.contains_key(&order.order.id) {
            return Err(format!("order not found: {}", order.order.id));
        }
        g.insert(order.order.id, order);
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<StoredOrder>, String> {
        Ok(self.by_id.read().await.values().cloned().collect())
    }

    async fn find_by_gateway_tx_id(&self, tx_id: &str) -> Result<Option<StoredOrder>, String> {
        Ok(self
            .by_id
            .read()
            .await
            .values()
            .find(|s| s.gateway_in_tx_id.as_deref() == Some(tx_id))
            .cloned())
    }
}

pub struct PostgresOrderRepository {
    pool: PgPool,
}

impl PostgresOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    fn map_row(
        id: Uuid,
        buyer_id: Uuid,
        seller_id: Uuid,
        amount: Decimal,
        status: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        custody_id: Option<Uuid>,
        anchor_tx_hash: Option<String>,
        fiat_rail: String,
        gateway_in_tx_id: Option<String>,
        funding_reference: Option<String>,
        pix_br_code: Option<String>,
        funding_instruction: Option<String>,
        risk_score: i32,
        risk_decision: String,
        description: Option<String>,
        off_ramp_tx_hash: Option<String>,
        brlx_escrow_transfer_tx_hash: Option<String>,
        soroban_escrow_contract_id: Option<String>,
        soroban_lock_tx_hash: Option<String>,
        soroban_mode: String,
    ) -> Result<StoredOrder, String> {
        if risk_score < 0 {
            return Err(format!("negative risk_score for order {id}"));
        }
        Ok(StoredOrder {
            order: Order {
                id,
                buyer_id,
                seller_id,
                amount: Money::new(amount),
                status: order_status_from_str(&status)?,
                created_at,
                updated_at,
            },
            custody_id,
            anchor_tx_hash,
            fiat_rail,
            gateway_in_tx_id,
            funding_reference,
            pix_br_code,
            funding_instruction,
            risk_score: risk_score as u32,
            risk_decision,
            description,
            off_ramp_tx_hash,
            brlx_escrow_transfer_tx_hash,
            soroban_escrow_contract_id,
            soroban_lock_tx_hash,
            soroban_mode,
        })
    }
}

#[async_trait]
impl OrderRepository for PostgresOrderRepository {
    async fn save(&self, stored: StoredOrder) -> Result<(), String> {
        let o = &stored.order;
        let risk_score =
            i32::try_from(stored.risk_score).map_err(|_| "risk_score overflow".to_string())?;
        sqlx::query(
            r#"
            INSERT INTO orders (
                id, buyer_id, seller_id, amount, status, created_at, updated_at,
                custody_id, anchor_tx_hash, fiat_rail, gateway_in_tx_id, funding_reference, pix_br_code, funding_instruction,
                risk_score, risk_decision, description,
                off_ramp_tx_hash, brlx_escrow_transfer_tx_hash, soroban_escrow_contract_id,
                soroban_lock_tx_hash, soroban_mode
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            "#,
        )
        .bind(o.id)
        .bind(o.buyer_id)
        .bind(o.seller_id)
        .bind(o.amount.decimal())
        .bind(o.status.to_string())
        .bind(o.created_at)
        .bind(o.updated_at)
        .bind(stored.custody_id)
        .bind(stored.anchor_tx_hash.as_deref())
        .bind(&stored.fiat_rail)
        .bind(stored.gateway_in_tx_id.as_deref())
        .bind(stored.funding_reference.as_deref())
        .bind(stored.pix_br_code.as_deref())
        .bind(stored.funding_instruction.as_deref())
        .bind(risk_score)
        .bind(&stored.risk_decision)
        .bind(stored.description.as_deref())
        .bind(stored.off_ramp_tx_hash.as_deref())
        .bind(stored.brlx_escrow_transfer_tx_hash.as_deref())
        .bind(stored.soroban_escrow_contract_id.as_deref())
        .bind(stored.soroban_lock_tx_hash.as_deref())
        .bind(&stored.soroban_mode)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<StoredOrder>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, buyer_id, seller_id, amount, status, created_at, updated_at,
                   custody_id, anchor_tx_hash, risk_score, risk_decision, description,
                   fiat_rail, gateway_in_tx_id, funding_reference, pix_br_code, funding_instruction,
                   off_ramp_tx_hash, brlx_escrow_transfer_tx_hash, soroban_escrow_contract_id,
                   soroban_lock_tx_hash, soroban_mode
            FROM orders
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let Some(r) = row else {
            return Ok(None);
        };

        Ok(Some(row_to_stored_order(&r)?))
    }

    async fn update(&self, stored: StoredOrder) -> Result<(), String> {
        let o = &stored.order;
        let risk_score =
            i32::try_from(stored.risk_score).map_err(|_| "risk_score overflow".to_string())?;
        let n = sqlx::query(
            r#"
            UPDATE orders SET
                buyer_id = $2,
                seller_id = $3,
                amount = $4,
                status = $5,
                created_at = $6,
                updated_at = $7,
                custody_id = $8,
                anchor_tx_hash = $9,
                fiat_rail = $10,
                gateway_in_tx_id = $11,
                funding_reference = $12,
                pix_br_code = $13,
                funding_instruction = $14,
                risk_score = $15,
                risk_decision = $16,
                description = $17,
                off_ramp_tx_hash = $18,
                brlx_escrow_transfer_tx_hash = $19,
                soroban_escrow_contract_id = $20,
                soroban_lock_tx_hash = $21,
                soroban_mode = $22
            WHERE id = $1
            "#,
        )
        .bind(o.id)
        .bind(o.buyer_id)
        .bind(o.seller_id)
        .bind(o.amount.decimal())
        .bind(o.status.to_string())
        .bind(o.created_at)
        .bind(o.updated_at)
        .bind(stored.custody_id)
        .bind(stored.anchor_tx_hash.as_deref())
        .bind(&stored.fiat_rail)
        .bind(stored.gateway_in_tx_id.as_deref())
        .bind(stored.funding_reference.as_deref())
        .bind(stored.pix_br_code.as_deref())
        .bind(stored.funding_instruction.as_deref())
        .bind(risk_score)
        .bind(&stored.risk_decision)
        .bind(stored.description.as_deref())
        .bind(stored.off_ramp_tx_hash.as_deref())
        .bind(stored.brlx_escrow_transfer_tx_hash.as_deref())
        .bind(stored.soroban_escrow_contract_id.as_deref())
        .bind(stored.soroban_lock_tx_hash.as_deref())
        .bind(&stored.soroban_mode)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?
        .rows_affected();

        if n == 0 {
            return Err(format!("order not found: {}", o.id));
        }
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<StoredOrder>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, buyer_id, seller_id, amount, status, created_at, updated_at,
                   custody_id, anchor_tx_hash, fiat_rail, gateway_in_tx_id, funding_reference, pix_br_code, funding_instruction,
                   risk_score, risk_decision, description,
                   off_ramp_tx_hash, brlx_escrow_transfer_tx_hash, soroban_escrow_contract_id,
                   soroban_lock_tx_hash, soroban_mode
            FROM orders
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            out.push(row_to_stored_order(&r)?);
        }
        Ok(out)
    }

    async fn find_by_gateway_tx_id(&self, tx_id: &str) -> Result<Option<StoredOrder>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, buyer_id, seller_id, amount, status, created_at, updated_at,
                   custody_id, anchor_tx_hash, fiat_rail, gateway_in_tx_id, funding_reference, pix_br_code, funding_instruction,
                   risk_score, risk_decision, description,
                   off_ramp_tx_hash, brlx_escrow_transfer_tx_hash, soroban_escrow_contract_id,
                   soroban_lock_tx_hash, soroban_mode
            FROM orders
            WHERE gateway_in_tx_id = $1
            LIMIT 1
            "#,
        )
        .bind(tx_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        row.map(|r| row_to_stored_order(&r)).transpose()
    }
}

fn row_to_stored_order(r: &sqlx::postgres::PgRow) -> Result<StoredOrder, String> {
    PostgresOrderRepository::map_row(
        r.try_get("id").map_err(|e| e.to_string())?,
        r.try_get("buyer_id").map_err(|e| e.to_string())?,
        r.try_get("seller_id").map_err(|e| e.to_string())?,
        r.try_get("amount").map_err(|e| e.to_string())?,
        r.try_get("status").map_err(|e| e.to_string())?,
        r.try_get("created_at").map_err(|e| e.to_string())?,
        r.try_get("updated_at").map_err(|e| e.to_string())?,
        r.try_get("custody_id").map_err(|e| e.to_string())?,
        r.try_get("anchor_tx_hash").map_err(|e| e.to_string())?,
        r.try_get("fiat_rail").map_err(|e| e.to_string())?,
        r.try_get("gateway_in_tx_id").map_err(|e| e.to_string())?,
        r.try_get("funding_reference").map_err(|e| e.to_string())?,
        r.try_get("pix_br_code").map_err(|e| e.to_string())?,
        r.try_get("funding_instruction")
            .map_err(|e| e.to_string())?,
        r.try_get("risk_score").map_err(|e| e.to_string())?,
        r.try_get("risk_decision").map_err(|e| e.to_string())?,
        r.try_get("description").map_err(|e| e.to_string())?,
        r.try_get("off_ramp_tx_hash").map_err(|e| e.to_string())?,
        r.try_get("brlx_escrow_transfer_tx_hash")
            .map_err(|e| e.to_string())?,
        r.try_get("soroban_escrow_contract_id")
            .map_err(|e| e.to_string())?,
        r.try_get("soroban_lock_tx_hash")
            .map_err(|e| e.to_string())?,
        r.try_get("soroban_mode").map_err(|e| e.to_string())?,
    )
}

fn order_status_from_str(s: &str) -> Result<OrderStatus, String> {
    match s {
        "draft" => Ok(OrderStatus::Draft),
        "pending_funding" => Ok(OrderStatus::PendingFunding),
        "funded" => Ok(OrderStatus::Funded),
        "in_custody" => Ok(OrderStatus::InCustody),
        "completed" => Ok(OrderStatus::Completed),
        "cancelled" => Ok(OrderStatus::Cancelled),
        "failed" => Ok(OrderStatus::Failed),
        _ => Err(format!("unknown order status: {s}")),
    }
}
