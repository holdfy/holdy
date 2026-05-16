//! Orquestra disputas, custódia e publicação de eventos.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::Mutex;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use apicash_custody::models::ReleaseConfirmation;
use apicash_custody::CustodyService;
use apicash_events::models::DisputeOpenedEvent;
use apicash_events::EventProducer;

use crate::error::DisputeError;
use crate::models::{Dispute, DisputeParty, DisputeStatus, Evidence, ResolutionType};
use crate::repository::DisputeRepository;
use crate::utils::DisputeTimeoutConfig;

/// Publica `DisputeOpened` (Pulsar ou no-op em testes).
#[async_trait]
pub trait DisputeEventSink: Send + Sync {
    async fn dispute_opened(&self, dispute: &Dispute) -> Result<(), DisputeError>;
}

/// Implementação via [`EventProducer`] (mutex para `&mut` publish).
pub struct PulsarDisputeEventSink {
    producer: Arc<Mutex<EventProducer>>,
}

impl PulsarDisputeEventSink {
    pub fn new(producer: Arc<Mutex<EventProducer>>) -> Self {
        Self { producer }
    }
}

#[async_trait]
impl DisputeEventSink for PulsarDisputeEventSink {
    async fn dispute_opened(&self, dispute: &Dispute) -> Result<(), DisputeError> {
        let mut g = self.producer.lock().await;
        g.publish_dispute_opened(DisputeOpenedEvent {
            dispute_id: dispute.id,
            order_id: dispute.order_id,
            opened_by: dispute.opened_by_user_id,
            opened_at: dispute.opened_at,
        })
        .await?;
        Ok(())
    }
}

/// Sem mensageria (testes / bootstrap).
pub struct NoopDisputeEventSink;

#[async_trait]
impl DisputeEventSink for NoopDisputeEventSink {
    async fn dispute_opened(&self, dispute: &Dispute) -> Result<(), DisputeError> {
        tracing::debug!(dispute_id = %dispute.id, "noop dispute event sink");
        Ok(())
    }
}

/// Serviço de aplicação: disputas + integração custódia + eventos.
pub struct DisputeService {
    repo: Arc<dyn DisputeRepository>,
    custody: Arc<CustodyService>,
    events: Arc<dyn DisputeEventSink>,
    timeout: DisputeTimeoutConfig,
}

impl DisputeService {
    pub fn new(
        repo: Arc<dyn DisputeRepository>,
        custody: Arc<CustodyService>,
        events: Arc<dyn DisputeEventSink>,
        timeout: DisputeTimeoutConfig,
    ) -> Self {
        Self {
            repo,
            custody,
            events,
            timeout,
        }
    }

    /// Abre disputa: trava custódia em `Disputed`, persiste e publica evento.
    #[instrument(skip(self, evidence), fields(order_id = %order_id))]
    pub async fn open_dispute(
        &self,
        order_id: Uuid,
        opened_by: DisputeParty,
        opened_by_user_id: Uuid,
        reason: String,
        evidence: Vec<Evidence>,
    ) -> Result<Dispute, DisputeError> {
        self.custody.mark_disputed(order_id).await?;

        let dispute = Dispute {
            id: Uuid::new_v4(),
            order_id,
            opened_by,
            opened_by_user_id,
            reason,
            status: DisputeStatus::Open,
            evidence,
            opened_at: Utc::now(),
            resolved_at: None,
            resolution_type: None,
            resolution_notes: None,
        };

        self.repo.insert(dispute.clone()).await?;
        self.events.dispute_opened(&dispute).await?;

        info!(dispute_id = %dispute.id, "dispute opened");
        Ok(dispute)
    }

    pub async fn get_dispute(&self, id: Uuid) -> Result<Option<Dispute>, DisputeError> {
        self.repo.get(id).await
    }

    pub async fn list_all_disputes(&self) -> Result<Vec<Dispute>, DisputeError> {
        self.repo.list_all().await
    }

    /// Resolve disputa e, quando aplicável, libera fundos na custódia (Stellar/Soroban espelhado depois).
    #[instrument(skip(self, notes))]
    pub async fn resolve_dispute(
        &self,
        dispute_id: Uuid,
        resolution: ResolutionType,
        notes: Option<String>,
    ) -> Result<(), DisputeError> {
        let mut d = self
            .repo
            .get(dispute_id)
            .await?
            .ok_or(DisputeError::NotFound(dispute_id))?;

        if matches!(d.status, DisputeStatus::Resolved | DisputeStatus::Closed) {
            return Err(DisputeError::InvalidState(format!(
                "dispute already finalized: {:?}",
                d.status
            )));
        }

        let now = Utc::now();
        d.resolved_at = Some(now);
        d.resolution_type = Some(resolution);
        d.resolution_notes = notes.clone();
        d.status = DisputeStatus::Resolved;

        match resolution {
            ResolutionType::Manual => {
                tracing::info!(
                    %dispute_id,
                    "manual resolution — escrow release handled outside automatic path"
                );
            }
            ResolutionType::Split => {
                warn!(
                    %dispute_id,
                    "split resolution — using full release path until Soroban split is wired"
                );
                self.release_custody_after_dispute(d.order_id, dispute_id)
                    .await?;
            }
            ResolutionType::RefundBuyer | ResolutionType::ReleaseToSeller => {
                self.release_custody_after_dispute(d.order_id, dispute_id)
                    .await?;
            }
        }

        self.repo.update(d).await?;
        Ok(())
    }

    async fn release_custody_after_dispute(
        &self,
        order_id: Uuid,
        dispute_id: Uuid,
    ) -> Result<(), DisputeError> {
        let confirmation = ReleaseConfirmation {
            released_by: Uuid::new_v4(),
            idempotency_key: format!("dispute-resolve-{dispute_id}"),
        };
        // Dispute resolution is an administrative override path and must not be blocked by the
        // buyer-only confirmation rule used for normal delivery confirmation.
        self.custody
            .release_funds_override(order_id, confirmation)
            .await?;
        Ok(())
    }

    /// Disputas abertas além do prazo: resolve como [`ResolutionType::Manual`] (sem liberação automática de escrow).
    #[instrument(skip(self))]
    pub async fn auto_resolve_timeout(&self) {
        let now = Utc::now();
        let open = match self.repo.list_open().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "list_open failed in auto_resolve_timeout");
                return;
            }
        };

        for d in open {
            if d.status != DisputeStatus::Open {
                continue;
            }
            if !self.timeout.is_past_deadline(d.opened_at, now) {
                continue;
            }

            tracing::warn!(
                dispute_id = %d.id,
                order_id = %d.order_id,
                "dispute past response window — escalating to Manual"
            );

            if let Err(e) = self
                .resolve_dispute(
                    d.id,
                    ResolutionType::Manual,
                    Some("auto timeout: escalated for manual review".into()),
                )
                .await
            {
                tracing::error!(error = %e, dispute_id = %d.id, "auto_resolve_timeout resolve failed");
            }
        }
    }
}
