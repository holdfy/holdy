//! Multi-provedor de rastreio com circuit breaker em cascata.
//!
//! Ordem de prioridade:
//!   1. Correios Business API (se `CORREIOS_USER` + `CORREIOS_ACCESS_CODE`)
//!   2. LinkTrack API       (se `LINKETRACK_USER` + `LINKETRACK_TOKEN`)
//!   3. Melhor Envio        (fallback sempre adicionado)

pub mod circuit_breaker;
pub mod correios;
pub mod linketrack;
pub mod melhor_envio;

use std::sync::Arc;

use async_trait::async_trait;
use tracing::{info, warn};

use crate::client::MelhorEnvioClient;
use crate::error::LogisticsError;
use crate::types::{TrackingInfo, TrackingStatus};

use circuit_breaker::CircuitBreaker;
use correios::CorreiosProvider;
use linketrack::LinkTrackProvider;
use melhor_envio::MelhorEnvioTracker;

// ─── Trait ────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait TrackingProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn track(&self, code: &str) -> Result<TrackingInfo, LogisticsError>;
}

// ─── Status helpers ───────────────────────────────────────────────────────────

/// Mapeia texto de descrição (português, case-insensitive) para `TrackingStatus`.
pub fn map_status_from_description(desc: &str) -> TrackingStatus {
    let lower = desc.to_lowercase();
    if lower.contains("entregue") || lower.contains("entrega realizada") {
        TrackingStatus::Delivered
    } else if lower.contains("saiu para entrega") || lower.contains("saiu p/ entrega") || lower.contains("em rota de entrega") {
        TrackingStatus::OutForDelivery
    } else if lower.contains("encaminhado")
        || lower.contains("em trânsito")
        || lower.contains("em transito")
        || lower.contains("trânsito")
        || lower.contains("transito")
        || lower.contains("objeto em trânsito")
        || lower.contains("objeto em transito")
    {
        TrackingStatus::InTransit
    } else if lower.contains("postado") || lower.contains("coletado") {
        TrackingStatus::Posted
    } else if lower.contains("devolvido") || lower.contains("retorno") {
        TrackingStatus::Returned
    } else if lower.contains("retornando") || lower.contains("devolvendo") {
        TrackingStatus::ReturnInProgress
    } else if lower.contains("problema") || lower.contains("impedimento") || lower.contains("avaria") {
        TrackingStatus::Exception
    } else {
        TrackingStatus::Unknown
    }
}

/// Mapeia `TrackingStatus` para rótulo em português.
pub fn status_label(s: &TrackingStatus) -> &'static str {
    match s {
        TrackingStatus::Posted => "Postado",
        TrackingStatus::InTransit => "Em trânsito",
        TrackingStatus::OutForDelivery => "Saiu para entrega",
        TrackingStatus::Delivered => "Entregue",
        TrackingStatus::ReturnInProgress => "Retorno em andamento",
        TrackingStatus::Returned => "Devolvido",
        TrackingStatus::Exception => "Problema na entrega",
        TrackingStatus::Unknown => "Status desconhecido",
    }
}

// ─── Entry wrapper ────────────────────────────────────────────────────────────

struct ProviderEntry {
    provider: Box<dyn TrackingProvider>,
    breaker: Arc<CircuitBreaker>,
}

// ─── CascadingTracker ─────────────────────────────────────────────────────────

/// Itera pelos provedores em prioridade, usando circuit breaker por provedor.
/// O primeiro a responder com sucesso ganha; os demais são pulados se abertos.
pub struct CascadingTracker {
    providers: Vec<ProviderEntry>,
}

impl CascadingTracker {
    /// Constrói o tracker a partir do ambiente.
    pub fn from_env(me_client: MelhorEnvioClient) -> Self {
        let mut providers: Vec<ProviderEntry> = Vec::new();

        // 1. Correios (se variáveis configuradas)
        if let Some(correios) = CorreiosProvider::from_env() {
            info!("tracking: Correios Business API disponível (CORREIOS_USER configurado)");
            let breaker = correios.breaker.clone();
            providers.push(ProviderEntry {
                provider: Box::new(correios),
                breaker,
            });
        }

        // 2. LinkTrack (se variáveis configuradas)
        if let Some(linketrack) = LinkTrackProvider::from_env() {
            info!("tracking: LinkTrack disponível (LINKETRACK_USER configurado)");
            let breaker = linketrack.breaker.clone();
            providers.push(ProviderEntry {
                provider: Box::new(linketrack),
                breaker,
            });
        }

        // 3. Melhor Envio (fallback, sempre adicionado)
        {
            let tracker = MelhorEnvioTracker::new(me_client);
            let breaker = tracker.breaker.clone();
            providers.push(ProviderEntry {
                provider: Box::new(tracker),
                breaker,
            });
        }

        Self { providers }
    }

    /// Tenta rastrear em cascata até o primeiro sucesso.
    pub async fn track(&self, raw_code: &str) -> Result<TrackingInfo, LogisticsError> {
        let code = raw_code.trim().to_uppercase();

        let mut last_error: Option<LogisticsError> = None;

        for entry in &self.providers {
            if !entry.breaker.is_available().await {
                warn!(
                    provider = %entry.provider.name(),
                    code = %code,
                    "tracking: provedor com circuit breaker aberto, pulando"
                );
                continue;
            }

            match entry.provider.track(&code).await {
                Ok(mut info) => {
                    entry.breaker.record_success().await;
                    info.provider_used = entry.provider.name().to_string();
                    info!( provider = %entry.provider.name(), code = %code, "tracking: sucesso");
                    return Ok(info);
                }
                Err(LogisticsError::TrackingNotFound(_)) => {
                    // Não é falha do provedor — código simplesmente não existe.
                    entry.breaker.record_success().await;
                    last_error = Some(LogisticsError::TrackingNotFound(code.clone()));
                    // Continua para o próximo provedor (pode ter o código em outro sistema).
                }
                Err(e) => {
                    warn!(
                        provider = %entry.provider.name(),
                        code = %code,
                        error = %e,
                        "tracking: falha, registrando no circuit breaker"
                    );
                    entry.breaker.record_failure().await;
                    last_error = Some(e);
                }
            }
        }

        // Se pelo menos um provedor disse "não encontrado", priorizar essa resposta.
        if let Some(LogisticsError::TrackingNotFound(_)) = last_error {
            return Err(LogisticsError::TrackingNotFound(code));
        }

        Err(LogisticsError::AllProvidersUnavailable(code))
    }
}
