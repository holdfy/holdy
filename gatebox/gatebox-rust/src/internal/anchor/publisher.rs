use std::error::Error;

use crate::internal::anchor::types::{EntityType, RequestPayload};
use crate::internal::anchor::validation::validate_request;
use crate::internal::anchor::{period, SCHEMA_VERSION};

/// Parâmetros para publicar um evento de ancoragem.
#[derive(Debug, Clone)]
pub struct PublishRequest {
    pub idempotency_key: String,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub payload_hash: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
    pub correlation_id: String,
    pub account_id: i64,
    pub customer_id: Option<i64>,
    pub company_id: Option<i64>,
    pub actor_document: String,
    pub actor_name: String,
    pub actor_type: String,
    pub client_ip: String,
    pub user_agent: String,
    pub metadata: Option<serde_json::Value>,
}

/// Interface para publicar pedidos de ancoragem (injeção e mock).
pub trait AnchorPublisher: Send + Sync {
    fn publish_anchor_request(&self, req: &PublishRequest) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn stop(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

/// Constrói RequestPayload a partir de PublishRequest (para Pulsar ou noop).
pub fn build_request_payload(req: &PublishRequest) -> Result<RequestPayload, Box<dyn Error + Send + Sync>> {
    validate_request(&req.entity_type, &req.entity_id, req.account_id)?;
    let (period_type, period_id) = period::period_from_time(req.occurred_at);
    Ok(RequestPayload {
        schema_version: SCHEMA_VERSION.to_string(),
        idempotency_key: req.idempotency_key.clone(),
        entity_type: req.entity_type.as_str().to_string(),
        entity_id: req.entity_id.clone(),
        payload_hash: req.payload_hash.clone(),
        occurred_at: req.occurred_at.to_rfc3339(),
        period_type: Some(period_type.as_str().to_string()),
        period_id: Some(period_id),
        correlation_id: if req.correlation_id.is_empty() {
            None
        } else {
            Some(req.correlation_id.clone())
        },
        account_id: req.account_id,
        customer_id: req.customer_id,
        company_id: req.company_id,
        actor_document: Some(req.actor_document.clone()).filter(|s| !s.is_empty()),
        actor_name: Some(req.actor_name.clone()).filter(|s| !s.is_empty()),
        actor_type: Some(req.actor_type.clone()).filter(|s| !s.is_empty()),
        client_ip: Some(req.client_ip.clone()).filter(|s| !s.is_empty()),
        user_agent: Some(req.user_agent.clone()).filter(|s| !s.is_empty()),
        metadata: req.metadata.clone(),
    })
}
