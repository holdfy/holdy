//! Consumer que processa fila de importação assíncrona de anúncios.

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use pulsar::{Consumer, SubType, TokioExecutor};

use crate::error::EventError;
use crate::models::{ApicashEvent, ImportCompletedEvent, ImportRequestedEvent, SUB_IMPORTER};
use crate::utils::PulsarClient;

#[async_trait]
pub trait ImporterPort: Send + Sync {
    /// Executa o scraping da URL e persiste o resultado.
    /// Retorna `(listing_id, None)` em sucesso, `(None, Some(error_msg))` em falha.
    async fn on_import_requested(
        &self,
        e: ImportRequestedEvent,
    ) -> Result<ImportCompletedEvent, EventError>;
}

pub async fn run_importer_consumer(
    client: &PulsarClient,
    handler: Arc<dyn ImporterPort>,
) -> Result<(), EventError> {
    let topic = client.main_topic();
    let mut consumer: Consumer<ApicashEvent, TokioExecutor> = client
        .inner
        .consumer()
        .with_topic(&topic)
        .with_consumer_name("apicash-importer-consumer")
        .with_subscription_type(SubType::Shared)
        .with_subscription(SUB_IMPORTER)
        .build()
        .await?;

    tracing::info!(%topic, subscription = SUB_IMPORTER, "importer consumer started");

    while let Some(res) = consumer.next().await {
        match res {
            Ok(msg) => {
                let ev = msg.deserialize();
                match ev {
                    ApicashEvent::ImportRequested(req) => {
                        let job_id = req.job_id;
                        match handler.on_import_requested(req).await {
                            Ok(completed) => {
                                tracing::info!(
                                    job_id = %job_id,
                                    listing_id = ?completed.listing_id,
                                    "importer: job concluído"
                                );
                            }
                            Err(e) => {
                                tracing::warn!(job_id = %job_id, error = %e, "importer: job falhou");
                            }
                        }
                    }
                    ApicashEvent::InvalidPayload(ref e) => {
                        tracing::warn!(error = %e.error, "importer: invalid payload");
                    }
                    _ => {}
                }
                consumer.ack(&msg).await?;
            }
            Err(e) => tracing::error!(error = %e, "importer consumer stream error"),
        }
    }

    Ok(())
}
