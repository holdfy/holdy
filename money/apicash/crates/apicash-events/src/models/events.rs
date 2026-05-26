//! Eventos de domínio serializados em JSON para o Pulsar.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use apicash_shared::{Money, OrderStatus};

/// Envelope principal publicado/consumido nos tópicos APICash.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ApicashEvent {
    OrderCreated(OrderCreatedEvent),
    PaymentReceived(PaymentReceivedEvent),
    ScoreCalculated(ScoreCalculatedEvent),
    FundsLocked(FundsLockedEvent),
    DeliveryConfirmed(DeliveryConfirmedEvent),
    YieldCalculated(YieldCalculatedEvent),
    FundsReleased(FundsReleasedEvent),
    DisputeOpened(DisputeOpenedEvent),
    TransactionRecorded(TransactionRecordedEvent),
    ReleaseRequested(ReleaseRequestedEvent),
    /// Fundos bloqueados no contrato Soroban (Stellar).
    FundsLockedOnChain(FundsLockedOnChainEvent),
    /// Yield distribuído on-chain (Soroban / ledger).
    YieldDistributedOnChain(YieldDistributedOnChainEvent),
    /// Liberação concluída on-chain.
    FundsReleasedOnChain(FundsReleasedOnChainEvent),
    /// Solicitação de importação assíncrona de anúncio (URL → ProductDraft).
    ImportRequested(ImportRequestedEvent),
    /// Resultado da importação (sucesso ou falha).
    ImportCompleted(ImportCompletedEvent),
    /// Payload JSON inválido (fallback do deserializador Pulsar).
    #[serde(rename = "invalid_payload")]
    InvalidPayload(InvalidPayloadEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidPayloadEvent {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreatedEvent {
    pub order_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub amount: Money,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentReceivedEvent {
    pub order_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub amount: Money,
    pub received_at: DateTime<Utc>,
    pub correlation_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreCalculatedEvent {
    pub user_id: Uuid,
    pub score: u32,
    pub decision: String,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsLockedEvent {
    pub order_id: Uuid,
    pub custody_id: Uuid,
    pub locked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryConfirmedEvent {
    pub order_id: Uuid,
    pub confirmed_by: Uuid,
    pub confirmed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldCalculatedEvent {
    pub custody_id: Uuid,
    pub order_id: Uuid,
    pub yield_pool: Money,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsReleasedEvent {
    pub order_id: Uuid,
    pub custody_id: Uuid,
    pub released_at: DateTime<Utc>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeOpenedEvent {
    pub dispute_id: Uuid,
    pub order_id: Uuid,
    pub opened_by: Uuid,
    pub opened_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecordedEvent {
    pub reference: String,
    pub order_id: Option<Uuid>,
    pub amount: Money,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseRequestedEvent {
    pub order_id: Uuid,
    pub requested_by: Uuid,
    pub requested_at: DateTime<Utc>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsLockedOnChainEvent {
    pub order_id: Uuid,
    pub custody_id: Uuid,
    pub escrow_contract_id: Option<String>,
    pub lock_tx_hash: Option<String>,
    pub brlx_escrow_transfer_tx_hash: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldDistributedOnChainEvent {
    pub order_id: Uuid,
    pub custody_id: Uuid,
    pub yield_pool: Money,
    pub seller_share: Money,
    pub buyer_cashback: Money,
    pub platform_share: Money,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsReleasedOnChainEvent {
    pub order_id: Uuid,
    pub custody_id: Uuid,
    pub release_tx_hash: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequestedEvent {
    /// UUID pré-gerado — identifica o job no banco antes do scraping.
    pub job_id: Uuid,
    pub url: String,
    pub user_id: Option<Uuid>,
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCompletedEvent {
    pub job_id: Uuid,
    pub listing_id: Option<Uuid>,
    pub success: bool,
    pub error_msg: Option<String>,
    pub completed_at: DateTime<Utc>,
}

impl PaymentReceivedEvent {
    /// Monta um [`apicash_shared::Order`] mínimo para `lock_funds`.
    pub fn to_order_pending(&self) -> apicash_shared::Order {
        apicash_shared::Order {
            id: self.order_id,
            buyer_id: self.buyer_id,
            seller_id: self.seller_id,
            amount: self.amount,
            status: OrderStatus::PendingFunding,
            created_at: self.received_at,
            updated_at: self.received_at,
        }
    }
}
