// PulsarAnchorPublisher - publica no tópico anchor-requests quando ANCHOR_PUBLISH_ENABLED=true
use std::error::Error;

use tokio::sync::mpsc;

use super::config::AnchorConfig;
use super::period;
use super::publisher::{AnchorPublisher, PublishRequest};
use super::types::RequestPayload;
use super::validation::validate_request;
use super::SCHEMA_VERSION;

pub struct PulsarAnchorPublisher {
    config: AnchorConfig,
    tx: mpsc::UnboundedSender<RequestPayload>,
}

impl PulsarAnchorPublisher {
    pub async fn new(config: AnchorConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let (tx, mut rx) = mpsc::unbounded_channel::<RequestPayload>();
        let topic = config.topic_full_name.clone();
        let url = config.pulsar_url.clone();

        tokio::spawn(async move {
            let client = match pulsar::Pulsar::builder(&url, pulsar::TokioExecutor).build().await {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("anchor pulsar client: {}", e);
                    return;
                }
            };
            let mut producer = match client
                .producer()
                .with_topic(topic)
                .with_name("anchor-publisher")
                .build()
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("anchor pulsar producer: {}", e);
                    return;
                }
            };
            while let Some(payload) = rx.recv().await {
                let body = match serde_json::to_vec(&payload) {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::error!("anchor marshal: {}", e);
                        continue;
                    }
                };
                let key = payload.idempotency_key.clone();
                let msg = pulsar::producer::Message {
                    payload: body.into(),
                    partition_key: Some(key),
                    ..Default::default()
                };
                match producer.send_non_blocking(msg).await {
                    Ok(fut) => {
                        if let Err(e) = fut.await {
                            tracing::error!("anchor publish receipt: {}", e);
                        }
                    }
                    Err(e) => tracing::error!("anchor publish: {}", e),
                }
            }
        });

        Ok(Self { config, tx })
    }
}

impl AnchorPublisher for PulsarAnchorPublisher {
    fn publish_anchor_request(&self, req: &PublishRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.config.publish_enabled {
            return Ok(());
        }

        validate_request(&req.entity_type, &req.entity_id, req.account_id)?;

        let (period_type, period_id) = period::period_from_time(req.occurred_at);

        let payload = RequestPayload {
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
        };

        self.tx.send(payload).map_err(|_| "anchor channel closed")?;
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
