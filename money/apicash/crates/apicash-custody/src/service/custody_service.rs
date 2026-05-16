//! Coordinates lock, yield accrual, and release. Com feature `soroban` + `APICASH_SOROBAN_ENABLED=1`,
//! invoca o contrato Soroban via [`crate::soroban_bridge`] (CLI `stellar`).

use std::sync::Arc;

use chrono::{Duration, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use uuid::Uuid;

use apicash_shared::{Order, DEFAULT_CUSTODY_DAYS};
use tracing::{info, instrument, warn};

use crate::errors::CustodyError;
use crate::models::{
    Custody, CustodyStatus, ReleaseConfirmation, ReleaseResult, YieldDistribution,
};
use crate::repository::CustodyRepository;
use crate::soroban_bridge::{
    custody_bridge_from_env, order_key_from_uuid, LockInvokeParams, SorobanCustodyBridge,
    SorobanDeployOutcome,
};
use crate::yield_logic::{split_yield_pool, YieldCalculator};

/// Application service for escrow custody and yield distribution.
pub struct CustodyService {
    repo: Arc<dyn CustodyRepository>,
    yield_calculator: YieldCalculator,
    soroban: Arc<dyn SorobanCustodyBridge>,
}

impl CustodyService {
    pub fn new(repo: Arc<dyn CustodyRepository>, yield_calculator: YieldCalculator) -> Self {
        Self::with_soroban_bridge(repo, yield_calculator, custody_bridge_from_env())
    }

    pub fn with_soroban_bridge(
        repo: Arc<dyn CustodyRepository>,
        yield_calculator: YieldCalculator,
        soroban: Arc<dyn SorobanCustodyBridge>,
    ) -> Self {
        Self {
            repo,
            yield_calculator,
            soroban,
        }
    }

    /// Deploy do Wasm do escrow (testnet) — delega ao bridge Soroban.
    pub async fn deploy_escrow_contract(&self) -> Result<SorobanDeployOutcome, CustodyError> {
        self.soroban.deploy_escrow_contract().await
    }

    /// Lock principal for an order (persistência + invocação Soroban opcional).
    #[instrument(skip(self, order), fields(order_id = %order.id))]
    pub async fn lock_funds(&self, order: &Order) -> Result<Custody, CustodyError> {
        info!(
            order_id = %order.id,
            buyer_id = %order.buyer_id,
            seller_id = %order.seller_id,
            amount = %order.amount,
            action = "FundsLocked",
            "custody: lock requested"
        );
        let locked_at = Utc::now();
        let expected = locked_at + Duration::days(i64::from(DEFAULT_CUSTODY_DAYS));
        let custody_id = Uuid::new_v4();
        let order_key = order_key_from_uuid(order.id);

        let buyer_stellar = std::env::var("APICASH_STELLAR_BUYER_ADDRESS")
            .unwrap_or_else(|_| "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACXK".into());
        let seller_stellar = std::env::var("APICASH_STELLAR_SELLER_ADDRESS")
            .unwrap_or_else(|_| "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACXK".into());
        let token_contract_id = std::env::var("APICASH_BRLX_TOKEN_CONTRACT_ID")
            .unwrap_or_else(|_| "CDXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXK2".into());
        let stroops = (order.amount.decimal() * Decimal::from(10_000_000i64))
            .trunc()
            .to_i128()
            .unwrap_or(0);

        let lock_out = self
            .soroban
            .invoke_lock(LockInvokeParams {
                order_id: order.id,
                order_key,
                buyer_stellar,
                seller_stellar,
                token_contract_id,
                amount_stroops: stroops,
            })
            .await?;

        let custody = Custody {
            id: custody_id,
            order_id: order.id,
            amount: order.amount,
            status: CustodyStatus::Locked,
            locked_at,
            expected_release_at: expected,
            actual_release_at: None,
            yield_earned: None,
            soroban_escrow_contract_id: lock_out.escrow_contract_id.clone(),
            soroban_is_mock: lock_out.is_mock,
            soroban_lock_tx_hash: lock_out.lock_tx_hash.clone(),
            soroban_release_tx_hash: None,
        };
        self.repo.insert(custody.clone()).await?;
        info!(
            custody_id = %custody.id,
            soroban_lock = ?custody.soroban_lock_tx_hash,
            action = "FundsLocked",
            success = true,
            "custody: funds locked"
        );
        Ok(custody)
    }

    /// Marca custódia como **Disputed** (funds permanecem travados; liberação automática bloqueada).
    pub async fn mark_disputed(&self, order_id: Uuid) -> Result<Custody, CustodyError> {
        let mut c = self
            .repo
            .get_by_order_id(order_id)
            .await?
            .ok_or(CustodyError::NotFound(order_id))?;

        if c.status != CustodyStatus::Locked {
            return Err(CustodyError::InvalidState(format!(
                "expected Locked to open dispute, got {:?}",
                c.status
            )));
        }

        // Best-effort on-chain dispute marker (feature `soroban` + env); mock bridge is a no-op.
        let order_key = order_key_from_uuid(order_id);
        if let Some(ref esc) = c.soroban_escrow_contract_id {
            match self.soroban.invoke_open_dispute(order_key, esc).await {
                Ok(Some(tx)) => {
                    tracing::info!(order_id = %order_id, tx = %tx, "soroban: dispute opened")
                }
                Ok(None) => {}
                Err(e) => tracing::warn!(error = %e, "soroban open_dispute skipped"),
            }
        }

        c.status = CustodyStatus::Disputed;
        self.repo.update(c.clone()).await?;
        Ok(c)
    }

    /// Accrue yield for `days` and return the 70/10/20 split of that yield pool (principal unchanged).
    pub async fn calculate_yield(
        &self,
        custody: &Custody,
        days: i64,
    ) -> Result<YieldDistribution, CustodyError> {
        info!(
            custody_id = %custody.id,
            order_id = %custody.order_id,
            %days,
            action = "YieldCalculated",
            "custody: yield calculation requested"
        );
        let accrued = self.yield_calculator.accrued_yield(custody.amount, days)?;
        let out = split_yield_pool(accrued)?;
        info!(
            custody_id = %custody.id,
            order_id = %custody.order_id,
            %days,
            yield_pool = %accrued,
            action = "YieldCalculated",
            success = true,
            "custody: yield calculated"
        );
        Ok(out)
    }

    /// Release custody **authorized by the buyer**.
    ///
    /// Security/business rule (critical): **only the buyer** (`order.buyer_id`) may confirm delivery
    /// and authorize releasing escrowed funds. Any attempt by the seller (or any other user) must be
    /// rejected.
    #[instrument(
        skip(self, order, confirmation),
        fields(order_id = %order.id, releasing_user_id = %releasing_user_id)
    )]
    pub async fn release_funds(
        &self,
        order: &Order,
        releasing_user_id: Uuid,
        confirmation: ReleaseConfirmation,
    ) -> Result<ReleaseResult, CustodyError> {
        info!(
            order_id = %order.id,
            releasing_user_id = %releasing_user_id,
            action = "FundsReleased",
            "custody: release requested"
        );
        if !order.is_buyer(&releasing_user_id) {
            return Err(CustodyError::UnauthorizedRelease);
        }
        // Defense-in-depth: the audit field must match the authenticated releasing user.
        if confirmation.released_by != releasing_user_id {
            return Err(CustodyError::Validation(
                "released_by must match the authenticated user".into(),
            ));
        }
        let out = self.release_funds_override(order.id, confirmation).await?;
        info!(
            order_id = %order.id,
            releasing_user_id = %releasing_user_id,
            action = "FundsReleased",
            success = true,
            "custody: release completed"
        );
        Ok(out)
    }

    /// Release custody **without buyer confirmation** (admin/dispute resolution only).
    ///
    /// This intentionally bypasses the "only buyer can release" rule and must never be exposed via
    /// buyer-facing confirmation endpoints.
    #[instrument(skip(self, confirmation), fields(order_id = %order_id, released_by = %confirmation.released_by))]
    pub async fn release_funds_override(
        &self,
        order_id: Uuid,
        confirmation: ReleaseConfirmation,
    ) -> Result<ReleaseResult, CustodyError> {
        info!(
            %order_id,
            released_by = %confirmation.released_by,
            action = "FundsReleased",
            "custody: release override requested"
        );
        let mut c = self
            .repo
            .get_by_order_id(order_id)
            .await?
            .ok_or(CustodyError::NotFound(order_id))?;

        if c.status != CustodyStatus::Locked && c.status != CustodyStatus::Disputed {
            return Err(CustodyError::InvalidState(format!(
                "expected Locked or Disputed, got {:?}",
                c.status
            )));
        }

        let now = Utc::now();
        let days = (now - c.locked_at).num_days().max(0);
        let yield_pool = self.yield_calculator.accrued_yield(c.amount, days)?;
        c.yield_earned = Some(yield_pool);
        let distribution = split_yield_pool(yield_pool)?;

        let order_key = order_key_from_uuid(order_id);
        if let Some(ref esc) = c.soroban_escrow_contract_id {
            match self.soroban.invoke_confirm_delivery(order_key, esc).await {
                Ok(Some(tx)) => {
                    tracing::info!(order_id = %order_id, tx = %tx, "soroban: confirm_delivery ok")
                }
                Ok(None) => {}
                Err(e) if soroban_strict() => return Err(e),
                Err(e) => warn!(error = %e, "soroban confirm_delivery skipped"),
            }
            match self.soroban.invoke_release(order_key, esc).await {
                Ok(Some(tx)) => c.soroban_release_tx_hash = Some(tx),
                Ok(None) => {}
                Err(e) if soroban_strict() => return Err(e),
                Err(e) => warn!(error = %e, "soroban release skipped"),
            }
        }

        c.status = CustodyStatus::Released;
        c.actual_release_at = Some(now);

        self.repo.update(c.clone()).await?;

        info!(
            custody_id = %c.id,
            released_by = %confirmation.released_by,
            yield_pool = %yield_pool,
            action = "FundsReleased",
            success = true,
            "custody: funds released"
        );

        Ok(ReleaseResult {
            custody_id: c.id,
            order_id: c.order_id,
            yield_distributed: distribution,
        })
    }

    /// Todas as custódias (relatórios / admin).
    pub async fn list_all_custodies(&self) -> Result<Vec<Custody>, CustodyError> {
        self.repo.list_all().await
    }
}

fn soroban_strict() -> bool {
    std::env::var("APICASH_SOROBAN_STRICT")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}
