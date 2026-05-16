//! Persistence for custody rows (Postgres via SQLx; in-memory for tests).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use tokio::sync::RwLock;
use uuid::Uuid;

use apicash_shared::Money;

use crate::errors::CustodyError;
use crate::models::{Custody, CustodyStatus};

#[async_trait]
pub trait CustodyRepository: Send + Sync {
    async fn insert(&self, custody: Custody) -> Result<(), CustodyError>;
    async fn get_by_order_id(&self, order_id: Uuid) -> Result<Option<Custody>, CustodyError>;
    async fn update(&self, custody: Custody) -> Result<(), CustodyError>;
    /// Painel admin / relatórios (in-memory ou SQL).
    async fn list_all(&self) -> Result<Vec<Custody>, CustodyError>;
}

/// In-memory store (replace with `sqlx::PgPool` + queries when wiring the service).
pub struct InMemoryCustodyRepository {
    by_order: Arc<RwLock<HashMap<Uuid, Custody>>>,
}

impl InMemoryCustodyRepository {
    pub fn new() -> Self {
        Self {
            by_order: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

impl Default for InMemoryCustodyRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CustodyRepository for InMemoryCustodyRepository {
    async fn insert(&self, custody: Custody) -> Result<(), CustodyError> {
        let mut g = self.by_order.write().await;
        if g.contains_key(&custody.order_id) {
            return Err(CustodyError::Validation(
                "custody already exists for order".into(),
            ));
        }
        g.insert(custody.order_id, custody);
        Ok(())
    }

    async fn get_by_order_id(&self, order_id: Uuid) -> Result<Option<Custody>, CustodyError> {
        Ok(self.by_order.read().await.get(&order_id).cloned())
    }

    async fn update(&self, custody: Custody) -> Result<(), CustodyError> {
        let mut g = self.by_order.write().await;
        if !g.contains_key(&custody.order_id) {
            return Err(CustodyError::NotFound(custody.order_id));
        }
        g.insert(custody.order_id, custody);
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Custody>, CustodyError> {
        Ok(self.by_order.read().await.values().cloned().collect())
    }
}

fn custody_status_to_str(s: CustodyStatus) -> &'static str {
    match s {
        CustodyStatus::Locked => "locked",
        CustodyStatus::Released => "released",
        CustodyStatus::Disputed => "disputed",
        CustodyStatus::Expired => "expired",
    }
}

fn custody_status_from_str(s: &str) -> Result<CustodyStatus, CustodyError> {
    match s {
        "locked" => Ok(CustodyStatus::Locked),
        "released" => Ok(CustodyStatus::Released),
        "disputed" => Ok(CustodyStatus::Disputed),
        "expired" => Ok(CustodyStatus::Expired),
        _ => Err(CustodyError::Validation(format!(
            "unknown custody status: {s}"
        ))),
    }
}

/// Repositório Postgres (SQLx). Requer migração `custody` aplicada (`sqlx migrate run`).
pub struct PostgresCustodyRepository {
    pool: PgPool,
}

impl PostgresCustodyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    fn map_row(
        id: Uuid,
        order_id: Uuid,
        amount: Decimal,
        status: String,
        locked_at: DateTime<Utc>,
        expected_release_at: DateTime<Utc>,
        actual_release_at: Option<DateTime<Utc>>,
        yield_earned: Option<Decimal>,
        soroban_escrow_contract_id: Option<String>,
        soroban_is_mock: bool,
        soroban_lock_tx_hash: Option<String>,
        soroban_release_tx_hash: Option<String>,
    ) -> Result<Custody, CustodyError> {
        Ok(Custody {
            id,
            order_id,
            amount: Money::new(amount),
            status: custody_status_from_str(&status)?,
            locked_at,
            expected_release_at,
            actual_release_at,
            yield_earned: yield_earned.map(Money::new),
            soroban_escrow_contract_id,
            soroban_is_mock,
            soroban_lock_tx_hash,
            soroban_release_tx_hash,
        })
    }
}

#[async_trait]
impl CustodyRepository for PostgresCustodyRepository {
    async fn insert(&self, custody: Custody) -> Result<(), CustodyError> {
        let y = custody.yield_earned.map(|m| m.decimal());
        sqlx::query(
            r#"
            INSERT INTO custody (
                id, order_id, amount, status, locked_at, expected_release_at, actual_release_at,
                yield_earned, soroban_escrow_contract_id, soroban_is_mock,
                soroban_lock_tx_hash, soroban_release_tx_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(custody.id)
        .bind(custody.order_id)
        .bind(custody.amount.decimal())
        .bind(custody_status_to_str(custody.status))
        .bind(custody.locked_at)
        .bind(custody.expected_release_at)
        .bind(custody.actual_release_at)
        .bind(y)
        .bind(custody.soroban_escrow_contract_id.as_deref())
        .bind(custody.soroban_is_mock)
        .bind(custody.soroban_lock_tx_hash.as_deref())
        .bind(custody.soroban_release_tx_hash.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| CustodyError::Repository(e.to_string()))?;
        Ok(())
    }

    async fn get_by_order_id(&self, order_id: Uuid) -> Result<Option<Custody>, CustodyError> {
        let row = sqlx::query(
            r#"
            SELECT id, order_id, amount, status, locked_at, expected_release_at, actual_release_at,
                   yield_earned, soroban_escrow_contract_id, soroban_is_mock,
                   soroban_lock_tx_hash, soroban_release_tx_hash
            FROM custody WHERE order_id = $1
            "#,
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CustodyError::Repository(e.to_string()))?;

        let Some(r) = row else {
            return Ok(None);
        };

        let id: Uuid = r
            .try_get("id")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let oid: Uuid = r
            .try_get("order_id")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let amount: Decimal = r
            .try_get("amount")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let status: String = r
            .try_get("status")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let locked_at: DateTime<Utc> = r
            .try_get("locked_at")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let expected_release_at: DateTime<Utc> = r
            .try_get("expected_release_at")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let actual_release_at: Option<DateTime<Utc>> = r
            .try_get("actual_release_at")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let yield_earned: Option<Decimal> = r
            .try_get("yield_earned")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let soroban_escrow_contract_id: Option<String> = r
            .try_get("soroban_escrow_contract_id")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let soroban_is_mock: bool = r
            .try_get("soroban_is_mock")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let soroban_lock_tx_hash: Option<String> = r
            .try_get("soroban_lock_tx_hash")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;
        let soroban_release_tx_hash: Option<String> = r
            .try_get("soroban_release_tx_hash")
            .map_err(|e| CustodyError::Repository(e.to_string()))?;

        Ok(Some(Self::map_row(
            id,
            oid,
            amount,
            status,
            locked_at,
            expected_release_at,
            actual_release_at,
            yield_earned,
            soroban_escrow_contract_id,
            soroban_is_mock,
            soroban_lock_tx_hash,
            soroban_release_tx_hash,
        )?))
    }

    async fn update(&self, custody: Custody) -> Result<(), CustodyError> {
        let y = custody.yield_earned.map(|m| m.decimal());
        let n = sqlx::query(
            r#"
            UPDATE custody SET
                amount = $2,
                status = $3,
                locked_at = $4,
                expected_release_at = $5,
                actual_release_at = $6,
                yield_earned = $7,
                soroban_escrow_contract_id = $8,
                soroban_is_mock = $9,
                soroban_lock_tx_hash = $10,
                soroban_release_tx_hash = $11
            WHERE order_id = $1
            "#,
        )
        .bind(custody.order_id)
        .bind(custody.amount.decimal())
        .bind(custody_status_to_str(custody.status))
        .bind(custody.locked_at)
        .bind(custody.expected_release_at)
        .bind(custody.actual_release_at)
        .bind(y)
        .bind(custody.soroban_escrow_contract_id.as_deref())
        .bind(custody.soroban_is_mock)
        .bind(custody.soroban_lock_tx_hash.as_deref())
        .bind(custody.soroban_release_tx_hash.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| CustodyError::Repository(e.to_string()))?
        .rows_affected();

        if n == 0 {
            return Err(CustodyError::NotFound(custody.order_id));
        }
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Custody>, CustodyError> {
        let rows = sqlx::query(
            r#"
            SELECT id, order_id, amount, status, locked_at, expected_release_at, actual_release_at,
                   yield_earned, soroban_escrow_contract_id, soroban_is_mock,
                   soroban_lock_tx_hash, soroban_release_tx_hash
            FROM custody
            ORDER BY locked_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CustodyError::Repository(e.to_string()))?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let id: Uuid = r
                .try_get("id")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let order_id: Uuid = r
                .try_get("order_id")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let amount: Decimal = r
                .try_get("amount")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let status: String = r
                .try_get("status")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let locked_at: DateTime<Utc> = r
                .try_get("locked_at")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let expected_release_at: DateTime<Utc> = r
                .try_get("expected_release_at")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let actual_release_at: Option<DateTime<Utc>> = r
                .try_get("actual_release_at")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let yield_earned: Option<Decimal> = r
                .try_get("yield_earned")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let soroban_escrow_contract_id: Option<String> = r
                .try_get("soroban_escrow_contract_id")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let soroban_is_mock: bool = r
                .try_get("soroban_is_mock")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let soroban_lock_tx_hash: Option<String> = r
                .try_get("soroban_lock_tx_hash")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;
            let soroban_release_tx_hash: Option<String> = r
                .try_get("soroban_release_tx_hash")
                .map_err(|e| CustodyError::Repository(e.to_string()))?;

            out.push(Self::map_row(
                id,
                order_id,
                amount,
                status,
                locked_at,
                expected_release_at,
                actual_release_at,
                yield_earned,
                soroban_escrow_contract_id,
                soroban_is_mock,
                soroban_lock_tx_hash,
                soroban_release_tx_hash,
            )?);
        }
        Ok(out)
    }
}
