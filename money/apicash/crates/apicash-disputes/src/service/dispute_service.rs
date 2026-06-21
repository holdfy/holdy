//! Orquestra disputas, custódia e publicação de eventos.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use tokio::sync::Mutex;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use apicash_custody::models::ReleaseConfirmation;
use apicash_custody::CustodyService;
use apicash_events::models::DisputeOpenedEvent;
use apicash_events::EventProducer;

use crate::error::DisputeError;
use crate::image_store::{hex_sha256, DisputeImageStore};
use crate::models::{
    AiVerdict, Dispute, DisputeParty, DisputeStatus, Evidence, EvidenceAnalysisResult,
    EvidenceKind, EvidenceParty, EvidenceRow, ResolutionType,
};
use crate::openai_client::OpenAiVisionClient;
use crate::repository::{DisputeRepository, EvidenceRepository};
use crate::utils::DisputeTimeoutConfig;

/// Prazo de submissão de evidências pelo comprador (horas).
const EVIDENCE_DEADLINE_HOURS: i64 = 48;
/// Threshold de confiança IA para decisão automática.
const AI_AUTO_RESOLVE_CONFIDENCE: f32 = 0.80;
/// Valor máximo (em centavos BRL) para auto-resolve pela IA.
const AI_AUTO_RESOLVE_MAX_BRL_CENTS: u64 = 200_000; // R$ 2.000,00

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
    repo:        Arc<dyn DisputeRepository>,
    evidence_repo: Arc<dyn EvidenceRepository>,
    custody:     Arc<CustodyService>,
    events:      Arc<dyn DisputeEventSink>,
    timeout:     DisputeTimeoutConfig,
    image_store: Option<Arc<DisputeImageStore>>,
    openai:      Option<Arc<OpenAiVisionClient>>,
}

impl DisputeService {
    pub fn new(
        repo:          Arc<dyn DisputeRepository>,
        evidence_repo: Arc<dyn EvidenceRepository>,
        custody:       Arc<CustodyService>,
        events:        Arc<dyn DisputeEventSink>,
        timeout:       DisputeTimeoutConfig,
    ) -> Self {
        let image_store = DisputeImageStore::from_env().map(Arc::new);
        let openai      = OpenAiVisionClient::from_env().map(Arc::new);
        Self { repo, evidence_repo, custody, events, timeout, image_store, openai }
    }

    /// Abre disputa: trava custódia em `Disputed`, persiste e publica evento.
    ///
    /// `buyer_score`: score atual do comprador (0-1000). Score < 200 → `high_risk_buyer = true`.
    #[instrument(skip(self, evidence), fields(order_id = %order_id))]
    pub async fn open_dispute(
        &self,
        order_id: Uuid,
        opened_by: DisputeParty,
        opened_by_user_id: Uuid,
        reason: String,
        evidence: Vec<Evidence>,
        buyer_score: Option<i32>,
    ) -> Result<Dispute, DisputeError> {
        self.custody.mark_disputed(order_id).await?;

        let now             = Utc::now();
        let high_risk_buyer = buyer_score.map(|s| s < 200).unwrap_or(false);

        let dispute = Dispute {
            id: Uuid::new_v4(),
            order_id,
            opened_by,
            opened_by_user_id,
            reason,
            status: DisputeStatus::Open,
            evidence,
            opened_at:       now,
            deadline_at:     Some(now + Duration::hours(EVIDENCE_DEADLINE_HOURS)),
            resolved_at:     None,
            resolution_type: None,
            resolution_notes: None,
            ai_verdict:      None,
            ai_confidence:   None,
            ai_reasoning:    None,
            high_risk_buyer,
        };

        self.repo.insert(dispute.clone()).await?;
        self.events.dispute_opened(&dispute).await?;

        if high_risk_buyer {
            warn!(dispute_id = %dispute.id, buyer_score = ?buyer_score, "high_risk_buyer flagged");
        }
        info!(dispute_id = %dispute.id, "dispute opened");
        Ok(dispute)
    }

    /// Retorna a disputa com as evidências persistidas na tabela `dispute_evidence`.
    pub async fn get_dispute_with_evidence(
        &self,
        dispute_id: Uuid,
    ) -> Result<Option<(Dispute, Vec<EvidenceRow>)>, DisputeError> {
        let Some(dispute) = self.repo.get(dispute_id).await? else {
            return Ok(None);
        };
        let rows = self.evidence_repo.list_for_dispute(dispute_id).await?;
        Ok(Some((dispute, rows)))
    }

    /// Adiciona evidência a uma disputa existente.
    /// `bytes`: conteúdo binário (foto/vídeo) ou `None` para textos.
    /// `content`: texto livre (rastreio, mensagem) ou `None` para mídias.
    #[instrument(skip(self, bytes), fields(dispute_id = %dispute_id))]
    pub async fn add_evidence(
        &self,
        dispute_id: Uuid,
        uploaded_by: Uuid,
        party: EvidenceParty,
        kind: EvidenceKind,
        ext: Option<&str>,
        bytes: Option<Vec<u8>>,
        content: Option<String>,
    ) -> Result<EvidenceRow, DisputeError> {
        let d = self.repo.get(dispute_id).await?
            .ok_or(DisputeError::NotFound(dispute_id))?;

        if matches!(d.status, DisputeStatus::Resolved | DisputeStatus::Closed) {
            return Err(DisputeError::InvalidState(
                "cannot add evidence to a resolved dispute".into(),
            ));
        }

        let (minio_key, minio_url, sha256) = if let (Some(b), Some(ext)) = (&bytes, ext) {
            if let Some(store) = &self.image_store {
                let (k, u, h) = store.upload(dispute_id, ext, b).await?;
                (Some(k), Some(u), h)
            } else {
                let h = hex_sha256(b);
                tracing::warn!(dispute_id = %dispute_id, "MinIO not configured — storing evidence hash only");
                (None, None, h)
            }
        } else {
            let text = content.as_deref().unwrap_or("");
            let h = hex_sha256(text.as_bytes());
            (None, None, h)
        };

        let row = EvidenceRow {
            id:          Uuid::new_v4(),
            dispute_id,
            uploaded_by,
            party,
            kind,
            minio_key,
            minio_url,
            content,
            sha256,
            ai_flagged:  false,
            created_at:  Utc::now(),
        };
        self.evidence_repo.insert(row.clone()).await?;
        info!(evidence_id = %row.id, kind = ?row.kind, "evidence added to dispute");
        Ok(row)
    }

    /// Analisa evidências com OpenAI Vision e, se confiança suficiente, auto-resolve.
    ///
    /// `order_amount_cents`: valor do pedido em centavos BRL (pedidos > R$2k → sempre manual).
    /// `listing_photo_urls`: URLs das fotos originais do anúncio (MinIO).
    /// `amount_str`: valor formatado (ex.: "150.00") para incluir na notificação WhatsApp.
    #[instrument(skip(self, listing_photo_urls, amount_str), fields(dispute_id = %dispute_id))]
    pub async fn analyze_and_maybe_resolve(
        &self,
        dispute_id: Uuid,
        order_amount_cents: u64,
        listing_photo_urls: Vec<String>,
        amount_str: String,
    ) -> Result<EvidenceAnalysisResult, DisputeError> {
        let evidence = self.evidence_repo.list_for_dispute(dispute_id).await?;
        let buyer_photos: Vec<String> = evidence.iter()
            .filter(|e| matches!(e.party, EvidenceParty::Buyer))
            .filter(|e| matches!(e.kind, EvidenceKind::Photo | EvidenceKind::Video))
            .filter_map(|e| e.minio_url.clone())
            .collect();

        let Some(openai) = &self.openai else {
            tracing::info!(dispute_id = %dispute_id, "OpenAI not configured — skipping AI analysis");
            return Ok(EvidenceAnalysisResult {
                verdict:    AiVerdict::Inconclusive,
                confidence: 0.0,
                reasoning:  "OpenAI not configured".into(),
                red_flags:  vec![],
            });
        };

        let result = openai.analyze(&listing_photo_urls, &buyer_photos).await
            .map_err(|e| DisputeError::InvalidState(format!("openai error: {e}")))?;

        // Persiste veredito na disputa.
        let mut d = self.repo.get(dispute_id).await?
            .ok_or(DisputeError::NotFound(dispute_id))?;
        d.ai_verdict    = Some(result.verdict);
        d.ai_confidence = Some(result.confidence);
        d.ai_reasoning  = Some(result.reasoning.clone());
        if !result.red_flags.is_empty() {
            // Marca evidências como flagged se IA detectou problemas.
            for ev in evidence.iter().filter(|e| matches!(e.party, EvidenceParty::Buyer)) {
                let _ = self.evidence_repo.mark_flagged(ev.id).await;
            }
        }

        // Auto-resolve se confiança alta, pedido não é grande, e comprador não é high risk.
        let can_auto = result.confidence >= AI_AUTO_RESOLVE_CONFIDENCE
            && order_amount_cents <= AI_AUTO_RESOLVE_MAX_BRL_CENTS
            && !d.high_risk_buyer;

        if can_auto {
            let resolution = match result.verdict {
                AiVerdict::FavorBuyer   => Some(ResolutionType::RefundBuyer),
                AiVerdict::FavorSeller  => Some(ResolutionType::ReleaseToSeller),
                AiVerdict::Inconclusive => None,
            };
            if let Some(res) = resolution {
                info!(
                    dispute_id = %dispute_id,
                    verdict = ?result.verdict,
                    confidence = result.confidence,
                    "AI auto-resolving dispute"
                );
                d.status = DisputeStatus::Resolved;
                d.resolved_at = Some(Utc::now());
                d.resolution_type = Some(res);
                d.resolution_notes = Some(format!(
                    "auto-resolved by AI (confidence={:.0}%): {}",
                    result.confidence * 100.0,
                    result.reasoning.chars().take(200).collect::<String>()
                ));
                self.repo.update(d.clone()).await?;
                self.release_custody_after_dispute(d.order_id, dispute_id).await?;
                // Finaliza order + off-ramp + notificação WA (fire-and-forget).
                let verdict_str = result.verdict.to_str().to_string();
                let order_id = d.order_id;
                tokio::spawn(async move {
                    finalize_dispute_order(order_id, &verdict_str).await;
                    notify_wa_dispute_resolved(order_id, &verdict_str, &amount_str).await;
                });
            }
        } else {
            d.status = DisputeStatus::UnderReview;
            self.repo.update(d).await?;
            info!(dispute_id = %dispute_id, confidence = result.confidence, "dispute moved to under_review (manual needed)");
        }

        Ok(result)
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

/// Chama `POST /orders/{id}/dispute/complete` no apicash-core.
/// Marca order como Completed e dispara off-ramp PIX ao ganhador.
async fn finalize_dispute_order(order_id: Uuid, verdict: &str) {
    let core_url = std::env::var("APICASH_CORE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let api_key = std::env::var("APICASH_API_KEY").unwrap_or_default();
    let body = serde_json::json!({ "verdict": verdict });
    let client = reqwest::Client::new();
    match client
        .post(format!("{core_url}/orders/{order_id}/dispute/complete"))
        .header("x-api-key", api_key)
        .json(&body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() =>
            tracing::info!(%order_id, verdict, "AI auto-resolve: order finalized + off-ramp triggered"),
        Ok(r) =>
            tracing::warn!(%order_id, status = %r.status(), "finalize_dispute_order: non-2xx"),
        Err(e) =>
            tracing::warn!(%order_id, error = %e, "finalize_dispute_order: http failed"),
    }
}

/// Notifica o serviço WhatsApp sobre resolução de disputa (fire-and-forget).
/// Usa `APICASH_WA_INTERNAL_URL` (padrão: http://127.0.0.1:3010).
async fn notify_wa_dispute_resolved(order_id: Uuid, verdict: &str, amount: &str) {
    let wa_url = std::env::var("APICASH_WA_INTERNAL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3010".to_string());
    let api_key = std::env::var("APICASH_API_KEY").unwrap_or_default();
    let body = serde_json::json!({
        "order_id": order_id,
        "verdict":  verdict,
        "amount":   amount,
    });
    let client = reqwest::Client::new();
    match client
        .post(format!("{wa_url}/internal/dispute-resolved"))
        .header("x-api-key", api_key)
        .json(&body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() =>
            tracing::info!(%order_id, verdict, "WA dispute-resolved notified"),
        Ok(r) =>
            tracing::warn!(%order_id, status = %r.status(), "WA dispute-resolved non-2xx"),
        Err(e) =>
            tracing::warn!(%order_id, error = %e, "WA dispute-resolved http failed"),
    }
}
