//! Fachada fina sobre [`DisputeService`] para handlers HTTP ou jobs.

use std::sync::Arc;

use uuid::Uuid;

use crate::error::DisputeError;
use crate::models::{Dispute, DisputeParty, Evidence};
use crate::service::DisputeService;

/// Agrupa chamadas ao serviço de disputas (reutilizável por Axum, workers, CLI).
pub struct DisputeHandler {
    service: Arc<DisputeService>,
}

impl DisputeHandler {
    pub fn new(service: Arc<DisputeService>) -> Self {
        Self { service }
    }

    pub async fn open_dispute(
        &self,
        order_id: Uuid,
        opened_by: DisputeParty,
        opened_by_user_id: Uuid,
        reason: String,
        evidence: Vec<Evidence>,
        buyer_score: Option<i32>,
    ) -> Result<Dispute, DisputeError> {
        self.service
            .open_dispute(order_id, opened_by, opened_by_user_id, reason, evidence, buyer_score)
            .await
    }
}
