//! Publicação tipada de eventos de domínio.

use pulsar::Producer;
use pulsar::TokioExecutor;

use crate::error::EventError;
use crate::models::{
    ApicashEvent, DeliveryConfirmedEvent, DisputeOpenedEvent, FundsLockedEvent,
    FundsLockedOnChainEvent, FundsReleasedEvent, FundsReleasedOnChainEvent, OrderCreatedEvent,
    PaymentReceivedEvent, ReleaseRequestedEvent, ScoreCalculatedEvent, TransactionRecordedEvent,
    YieldCalculatedEvent, YieldDistributedOnChainEvent,
};

/// Producer APICash com métodos por tipo de evento.
pub struct EventProducer {
    producer: Producer<TokioExecutor>,
}

impl EventProducer {
    pub async fn new(client: &crate::utils::PulsarClient, topic: &str) -> Result<Self, EventError> {
        let producer = client
            .inner
            .producer()
            .with_topic(topic)
            .with_name("apicash-event-producer")
            .build()
            .await?;
        tracing::info!(%topic, "event producer ready");
        Ok(Self { producer })
    }

    async fn send(&mut self, event: ApicashEvent) -> Result<(), EventError> {
        tracing::debug!(?event, "pulsar publish");
        let fut = self.producer.send_non_blocking(event).await?;
        fut.await?;
        Ok(())
    }

    pub async fn publish_order_created(
        &mut self,
        event: OrderCreatedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(order_id = %event.order_id, "publish OrderCreated");
        self.send(ApicashEvent::OrderCreated(event)).await
    }

    pub async fn publish_payment_received(
        &mut self,
        event: PaymentReceivedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(order_id = %event.order_id, "publish PaymentReceived");
        self.send(ApicashEvent::PaymentReceived(event)).await
    }

    pub async fn publish_score_calculated(
        &mut self,
        event: ScoreCalculatedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(user_id = %event.user_id, score = event.score, "publish ScoreCalculated");
        self.send(ApicashEvent::ScoreCalculated(event)).await
    }

    pub async fn publish_funds_locked(
        &mut self,
        event: FundsLockedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(order_id = %event.order_id, custody_id = %event.custody_id, "publish FundsLocked");
        self.send(ApicashEvent::FundsLocked(event)).await
    }

    pub async fn publish_delivery_confirmed(
        &mut self,
        event: DeliveryConfirmedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(order_id = %event.order_id, "publish DeliveryConfirmed");
        self.send(ApicashEvent::DeliveryConfirmed(event)).await
    }

    pub async fn publish_yield_calculated(
        &mut self,
        event: YieldCalculatedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(custody_id = %event.custody_id, "publish YieldCalculated");
        self.send(ApicashEvent::YieldCalculated(event)).await
    }

    pub async fn publish_funds_released(
        &mut self,
        event: FundsReleasedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(order_id = %event.order_id, "publish FundsReleased");
        self.send(ApicashEvent::FundsReleased(event)).await
    }

    pub async fn publish_dispute_opened(
        &mut self,
        event: DisputeOpenedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(dispute_id = %event.dispute_id, "publish DisputeOpened");
        self.send(ApicashEvent::DisputeOpened(event)).await
    }

    pub async fn publish_transaction_recorded(
        &mut self,
        event: TransactionRecordedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(reference = %event.reference, "publish TransactionRecorded");
        self.send(ApicashEvent::TransactionRecorded(event)).await
    }

    pub async fn publish_release_requested(
        &mut self,
        event: ReleaseRequestedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(order_id = %event.order_id, "publish ReleaseRequested");
        self.send(ApicashEvent::ReleaseRequested(event)).await
    }

    pub async fn publish_funds_locked_on_chain(
        &mut self,
        event: FundsLockedOnChainEvent,
    ) -> Result<(), EventError> {
        tracing::info!(order_id = %event.order_id, "publish FundsLockedOnChain");
        self.send(ApicashEvent::FundsLockedOnChain(event)).await
    }

    pub async fn publish_yield_distributed_on_chain(
        &mut self,
        event: YieldDistributedOnChainEvent,
    ) -> Result<(), EventError> {
        tracing::info!(order_id = %event.order_id, "publish YieldDistributedOnChain");
        self.send(ApicashEvent::YieldDistributedOnChain(event))
            .await
    }

    pub async fn publish_funds_released_on_chain(
        &mut self,
        event: FundsReleasedOnChainEvent,
    ) -> Result<(), EventError> {
        tracing::info!(order_id = %event.order_id, "publish FundsReleasedOnChain");
        self.send(ApicashEvent::FundsReleasedOnChain(event)).await
    }
}
