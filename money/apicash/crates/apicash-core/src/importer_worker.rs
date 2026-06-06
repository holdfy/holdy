//! Worker que consome a fila Pulsar de importação assíncrona.
//!
//! Implementa [`apicash_events::ImporterPort`]: recebe `ImportRequested`,
//! chama `ImporterService::import`, salva no Postgres via `ListingRepository`
//! e retorna `ImportCompletedEvent`.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use apicash_events::{EventError, ImportCompletedEvent, ImportRequestedEvent, ImporterPort};
use apicash_importer::ImporterService;

use crate::repository::ListingRepository;

pub struct ImporterWorker {
    pub importer: Arc<ImporterService>,
    pub repo: Arc<ListingRepository>,
}

#[async_trait]
impl ImporterPort for ImporterWorker {
    async fn on_import_requested(
        &self,
        e: ImportRequestedEvent,
    ) -> Result<ImportCompletedEvent, EventError> {
        match self.importer.import(&e.url).await {
            Ok(draft) => match self.repo.save(&draft, e.user_id, None).await {
                Ok(listing_id) => {
                    let _ = self.repo.complete_import_job(e.job_id, listing_id).await;
                    Ok(ImportCompletedEvent {
                        job_id: e.job_id,
                        listing_id: Some(listing_id),
                        success: true,
                        error_msg: None,
                        completed_at: Utc::now(),
                    })
                }
                Err(db_err) => {
                    let msg = db_err.to_string();
                    let _ = self.repo.fail_import_job(e.job_id, &msg).await;
                    Ok(ImportCompletedEvent {
                        job_id: e.job_id,
                        listing_id: None,
                        success: false,
                        error_msg: Some(msg),
                        completed_at: Utc::now(),
                    })
                }
            },
            Err(import_err) => {
                let msg = import_err.to_string();
                let _ = self.repo.fail_import_job(e.job_id, &msg).await;
                Ok(ImportCompletedEvent {
                    job_id: e.job_id,
                    listing_id: None,
                    success: false,
                    error_msg: Some(msg),
                    completed_at: Utc::now(),
                })
            }
        }
    }
}

/// Spawna o consumer Pulsar em background. No-op se Pulsar não estiver configurado.
pub async fn maybe_spawn_importer_consumer(
    importer: Arc<ImporterService>,
    repo: Option<Arc<ListingRepository>>,
) {
    let pulsar_url = match std::env::var("APICASH_PULSAR__SERVICE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        Some(u) => u,
        None => {
            tracing::debug!("importer_worker: APICASH_PULSAR__SERVICE_URL não configurado, fila async desativada");
            return;
        }
    };

    let Some(repo) = repo else {
        tracing::warn!("importer_worker: ListingRepository indisponível (DATABASE_URL?), fila async desativada");
        return;
    };

    let cfg = apicash_events::config::PulsarConfig::from_env();
    tokio::spawn(async move {
        match apicash_events::PulsarClient::connect(cfg).await {
            Ok(client) => {
                let worker = Arc::new(ImporterWorker { importer, repo });
                tracing::info!(%pulsar_url, "importer_worker: consumer iniciado");
                if let Err(e) = apicash_events::run_importer_consumer(&client, worker).await {
                    tracing::error!(error = %e, "importer_worker: consumer encerrado com erro");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "importer_worker: falha ao conectar ao Pulsar");
            }
        }
    });
}

/// Spawna o consumer NATS JetStream em background. No-op se NATS_URL não estiver configurado.
pub async fn maybe_spawn_importer_consumer_nats(
    importer: Arc<ImporterService>,
    repo: Option<Arc<ListingRepository>>,
) {
    let nats_url = match std::env::var("NATS_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        Some(u) => u,
        None => {
            tracing::debug!("importer_worker: NATS_URL não configurado, fila async desativada");
            return;
        }
    };

    let Some(repo) = repo else {
        tracing::warn!("importer_worker: ListingRepository indisponível, fila NATS async desativada");
        return;
    };

    tokio::spawn(async move {
        let worker = Arc::new(ImporterWorker { importer, repo });
        tracing::info!(%nats_url, "importer_worker: consumer NATS iniciado");
        if let Err(e) = apicash_events::run_importer_consumer_nats(&nats_url, worker).await {
            tracing::error!(error = %e, "importer_worker: consumer NATS encerrado com erro");
        }
    });
}
