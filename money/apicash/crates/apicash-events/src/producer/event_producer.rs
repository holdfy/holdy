//! Publicação tipada de eventos de domínio.
//! Suporta dois backends via enum dispatch: Pulsar (padrão) e NATS JetStream.

use async_nats::jetstream;
use pulsar::Producer;
use pulsar::TokioExecutor;

use crate::error::EventError;
use crate::models::{
    ApicashEvent, DeliveryConfirmedEvent, DisputeOpenedEvent, FundsLockedEvent,
    FundsLockedOnChainEvent, FundsReleasedEvent, FundsReleasedOnChainEvent, ImportCompletedEvent,
    ImportRequestedEvent, OrderCreatedEvent, PaymentReceivedEvent, ReleaseRequestedEvent,
    ScoreCalculatedEvent, TransactionRecordedEvent, YieldCalculatedEvent,
    YieldDistributedOnChainEvent,
};

enum Inner {
    Pulsar(Producer<TokioExecutor>),
    Nats { context: jetstream::Context, subject: String },
}

/// Producer APICash com métodos por tipo de evento.
/// Seleciona backend (Pulsar ou NATS) internamente — call sites não mudam.
pub struct EventProducer {
    inner: Inner,
}

impl EventProducer {
    /// Cria producer Pulsar (comportamento original).
    pub async fn new(client: &crate::utils::PulsarClient, topic: &str) -> Result<Self, EventError> {
        let producer = client
            .inner
            .producer()
            .with_topic(topic)
            .with_name("apicash-event-producer")
            .build()
            .await?;
        tracing::info!(%topic, "event producer ready (pulsar)");
        Ok(Self { inner: Inner::Pulsar(producer) })
    }

    /// Cria producer NATS JetStream.
    /// Cria o stream `APICASH_EVENTS` se não existir.
    pub async fn new_nats(nats_url: &str, subject: impl Into<String>) -> Result<Self, EventError> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| EventError::Nats(e.to_string()))?;
        let context = jetstream::new(client);
        let subject = subject.into();
        context
            .get_or_create_stream(jetstream::stream::Config {
                name: "APICASH_EVENTS".to_string(),
                subjects: vec!["apicash.events".to_string()],
                retention: jetstream::stream::RetentionPolicy::WorkQueue,
                max_age: std::time::Duration::from_secs(7 * 24 * 3600),
                ..Default::default()
            })
            .await
            .map_err(|e| EventError::Nats(e.to_string()))?;
        tracing::info!(%nats_url, %subject, "event producer ready (nats)");
        Ok(Self { inner: Inner::Nats { context, subject } })
    }

    async fn send(&mut self, event: ApicashEvent) -> Result<(), EventError> {
        match &mut self.inner {
            Inner::Pulsar(producer) => {
                tracing::debug!(?event, "pulsar publish");
                let fut = producer.send_non_blocking(event).await?;
                fut.await?;
            }
            Inner::Nats { context, subject } => {
                tracing::debug!(subject, "nats publish");
                let payload = serde_json::to_vec(&event)
                    .map_err(|e| EventError::Serialization(e.to_string()))?;
                context
                    .publish(subject.clone(), payload.into())
                    .await
                    .map_err(|e| EventError::Nats(e.to_string()))?
                    .await
                    .map_err(|e| EventError::Nats(e.to_string()))?;
            }
        }
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

    pub async fn publish_import_requested(
        &mut self,
        event: ImportRequestedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(job_id = %event.job_id, url = %event.url, "publish ImportRequested");
        self.send(ApicashEvent::ImportRequested(event)).await
    }

    pub async fn publish_import_completed(
        &mut self,
        event: ImportCompletedEvent,
    ) -> Result<(), EventError> {
        tracing::info!(job_id = %event.job_id, success = event.success, "publish ImportCompleted");
        self.send(ApicashEvent::ImportCompleted(event)).await
    }
}
