//! Cliente Pulsar tipado + integração `SerializeMessage` / `DeserializeMessage`.

use std::sync::Arc;

use pulsar::message::Payload;
use pulsar::producer;
use pulsar::{DeserializeMessage, Error as PulsarError, Pulsar, SerializeMessage, TokioExecutor};

use crate::config::PulsarConfig;
use crate::error::EventError;
use crate::models::ApicashEvent;

/// Cliente compartilhado para criar producers e consumers.
#[derive(Clone)]
pub struct PulsarClient {
    pub inner: Pulsar<TokioExecutor>,
    pub config: Arc<PulsarConfig>,
}

impl PulsarClient {
    /// Conecta ao broker configurado em [`PulsarConfig::service_url`].
    pub async fn connect(cfg: PulsarConfig) -> Result<Self, EventError> {
        let url = cfg.service_url.clone();
        let inner = Pulsar::builder(url, TokioExecutor).build().await?;
        tracing::info!(
            service_url = %cfg.service_url,
            tenant = %cfg.tenant,
            namespace = %cfg.namespace,
            "pulsar client connected"
        );
        Ok(Self {
            inner,
            config: Arc::new(cfg),
        })
    }

    /// Tópico principal de domínio.
    pub fn main_topic(&self) -> String {
        self.config.main_topic()
    }
}

impl SerializeMessage for ApicashEvent {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for ApicashEvent {
    type Output = ApicashEvent;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data).unwrap_or_else(|e| {
            tracing::error!(error = %e, "failed to deserialize ApicashEvent");
            ApicashEvent::InvalidPayload(crate::models::InvalidPayloadEvent {
                error: e.to_string(),
            })
        })
    }
}
