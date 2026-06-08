//! Orquestra mensagens de texto e o fluxo de pedido.

use std::sync::Arc;

use apicash_auth::{AuthConfig, AuthService, Role};
use apicash_logistics::LogisticsService;
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::conversation_store::{ConversationStore, MessageDirection, SummaryTrigger, WaMessage};

use crate::core_api::{CoreApiClient, CoreApiError};
use crate::handlers::holdfy::{
    self, next_collect_step, parse_holdfy_message, parse_loose_fields, HoldfyCollectStep,
};
use crate::handlers::order_flow;
use crate::models::WhatsAppEvent;
use crate::outbound::Outbound;
use crate::payment_notify::{OrderPaymentParties, PaymentNotifyRegistry};
use crate::handlers::holdfy::phone::peer_lookup_digit_variants;
use crate::session::{
    CreatingOrderStep, OrderDraft, OrderFlowState, SessionManager, UserSession,
};
use crate::wa_peer::canonical_whatsapp_peer_digits;
use crate::utils::masking::mask_whatsapp_peer;
use crate::utils::message_templates;
use crate::utils::qr_code;


pub struct MessageHandler {
    core: CoreApiClient,
    outbound: Arc<Outbound>,
    sessions: Arc<SessionManager>,
    payment_registry: Arc<PaymentNotifyRegistry>,
    jwt: AuthService,
    conv_store: Arc<ConversationStore>,
    logistics: Arc<LogisticsService>,
    pg_pool: Option<Arc<sqlx::PgPool>>,
}

impl MessageHandler {
    pub fn new(
        core: CoreApiClient,
        outbound: Arc<Outbound>,
        sessions: Arc<SessionManager>,
        payment_registry: Arc<PaymentNotifyRegistry>,
        conv_store: Arc<ConversationStore>,
        logistics: Arc<LogisticsService>,
    ) -> Self {
        Self {
            core,
            outbound,
            sessions,
            payment_registry,
            jwt: AuthService::new(AuthConfig::from_env()),
            conv_store,
            logistics,
            pg_pool: None,
        }
    }

    pub fn with_pg_pool(mut self, pool: sqlx::PgPool) -> Self {
        self.pg_pool = Some(Arc::new(pool));
        self
    }

    /// Persiste uma mensagem no MongoDB (fire-and-forget).
    fn record_inbound(&self, ev: &WhatsAppEvent, flow_state_tag: &str, order_id: Option<Uuid>) {
        let store = self.conv_store.clone();
        let msg = WaMessage {
            session_key: ev.sender_id.clone(),
            user_id: crate::session::user_id_for_peer_key(&ev.sender_id).to_string(),
            direction: MessageDirection::Inbound,
            body: ev.body.clone(),
            timestamp: Utc::now(),
            order_id: order_id.map(|id| id.to_string()),
            flow_state_tag: flow_state_tag.to_string(),
            message_id: Some(ev.message_id.clone()),
        };
        tokio::spawn(async move { store.record_message(msg).await });
    }

    #[allow(dead_code)]
    fn record_outbound(&self, peer: &str, body: &str, order_id: Option<Uuid>) {
        let store = self.conv_store.clone();
        let user_id = crate::session::user_id_for_peer_key(peer).to_string();
        let msg = WaMessage {
            session_key: peer.to_string(),
            user_id,
            direction: MessageDirection::Outbound,
            body: body.to_string(),
            timestamp: Utc::now(),
            order_id: order_id.map(|id| id.to_string()),
            flow_state_tag: "outbound".to_string(),
            message_id: None,
        };
        tokio::spawn(async move { store.record_message(msg).await });
    }

    /// Registra o código de rastreio do vendedor, notifica o comprador e confirma ao vendedor.
    /// Chamado quando o vendedor envia o código após pagamento confirmado.
    async fn register_seller_tracking(&self, seller_peer: &str, code: &str, order_id: Uuid) {
        // Busca o peer do comprador no registro de pagamentos.
        let buyer_peer = self
            .payment_registry
            .get(order_id)
            .await
            .map(|p| p.buyer_peer);

        let buyer_peer_ref = buyer_peer.as_deref().unwrap_or("");

        // Persiste na tabela de monitoramento para polling proativo.
        if let Some(pool) = &self.pg_pool {
            if let Err(e) = crate::tracking_monitor::upsert_tracking(
                pool,
                order_id,
                code,
                buyer_peer_ref,
                seller_peer,
            )
            .await
            {
                tracing::warn!(
                    order_id = %order_id,
                    code = %code,
                    error = %e,
                    "register_seller_tracking: falha ao persistir código (monitoramento não ativo)"
                );
            }
        }

        // Notifica o comprador com o código de rastreio.
        if !buyer_peer_ref.is_empty() {
            self.outbound
                .send_text(
                    buyer_peer_ref,
                    &message_templates::buyer_order_shipped(code, &order_id),
                )
                .await;
            tracing::info!(
                order_id = %order_id,
                code = %code,
                buyer = %mask_whatsapp_peer(buyer_peer_ref),
                "register_seller_tracking: comprador notificado"
            );
        } else {
            tracing::warn!(
                order_id = %order_id,
                code = %code,
                "register_seller_tracking: buyer_peer não encontrado, comprador não notificado"
            );
        }

        // Confirma ao vendedor que o código foi registrado.
        self.outbound
            .send_text(seller_peer, &message_templates::seller_tracking_registered(code))
            .await;
    }

    /// Responde a um pedido de rastreio via WhatsApp.
    async fn handle_tracking_request(&self, peer: &str, code: &str, _order_id: Option<Uuid>) {
        use apicash_logistics::LogisticsError;

        match self.logistics.track(code).await {
            Ok(info) => {
                let status_str = apicash_logistics::tracking::status_label(&info.current_status);
                let last_event = info
                    .events
                    .first()
                    .map(|e| e.description.as_str())
                    .unwrap_or("—");
                let msg = message_templates::tracking_result(
                    code,
                    status_str,
                    last_event,
                    &info.provider_used,
                );
                self.outbound.send_text(peer, &msg).await;
            }
            Err(LogisticsError::TrackingNotFound(_)) => {
                self.outbound
                    .send_text(peer, &message_templates::tracking_not_found(code))
                    .await;
            }
            Err(LogisticsError::AllProvidersUnavailable(_)) => {
                self.outbound
                    .send_text(peer, message_templates::tracking_all_providers_down())
                    .await;
            }
            Err(e) => {
                tracing::warn!(peer = %mask_whatsapp_peer(peer), code = %code, error = %e, "tracking: falha inesperada");
                self.outbound
                    .send_text(peer, message_templates::tracking_all_providers_down())
                    .await;
            }
        }
    }

    fn trigger_summary(&self, session_key: &str, user_id: Uuid, order_id: Option<Uuid>, trigger: SummaryTrigger) {
        let store = self.conv_store.clone();
        let sk = session_key.to_string();
        tokio::spawn(async move {
            store.generate_and_save_summary(&sk, user_id, order_id, trigger).await;
        });
    }

    /// Pagamento confirmado no Gatebox: avisa vendedor e comprador e dispara settle (BRLx → testnet).
    pub async fn notify_bank_payment(
        &self,
        order_id: Uuid,
    ) -> Result<(), String> {
        if self.payment_registry.get(order_id).await.is_none() {
            if let Some(p) = self.sessions.find_parties_by_order_id(order_id).await {
                self.payment_registry.register(order_id, p).await;
            }
        }

        let parties = match self.payment_registry.get(order_id).await {
            Some(p) => p,
            None => {
                return Err(format!(
                    "order {order_id}: parties not found for WhatsApp notify"
                ));
            }
        };

        if self.payment_registry.was_notified(order_id).await {
            tracing::info!(%order_id, "bank payment notify: already sent, skipping");
            return Ok(());
        }

        let msg = message_templates::payment_completed_notify(&order_id, &parties.amount);
        self.outbound.send_text(&parties.seller_peer, &msg).await;
        self.outbound.send_text(&parties.buyer_peer, &msg).await;

        let tracking_wait = message_templates::awaiting_seller_tracking_code(&order_id);
        self.outbound
            .send_text(&parties.seller_peer, &tracking_wait)
            .await;

        self.payment_registry.mark_notified(order_id).await;

        // Dispara settle em background: poll anchor → BRLx real no testnet → Soroban lock.
        // Fire-and-forget: não bloqueia o notify; erros são logados.
        {
            let core = self.core.clone();
            tokio::spawn(async move {
                match core.settle_order_internal(order_id, None).await {
                    Ok(_) => tracing::info!(%order_id, "settle_order_internal: BRLx bloqueado no Soroban"),
                    Err(e) => tracing::warn!(%order_id, error = %e, "settle_order_internal: falhou (rail simulated ignora, anchor requer testnet configurado)"),
                }
            });
        }

        // Gerar resumo quando pagamento confirmado
        let buyer_uid = crate::session::user_id_for_peer_key(&parties.buyer_peer);
        let seller_uid = crate::session::user_id_for_peer_key(&parties.seller_peer);
        self.trigger_summary(&parties.buyer_peer, buyer_uid, Some(order_id), SummaryTrigger::PaymentConfirmed);
        self.trigger_summary(&parties.seller_peer, seller_uid, Some(order_id), SummaryTrigger::PaymentConfirmed);

        // Comprador entra em AwaitingConfirmation — aguarda "recebi" para liberar escrow.
        let mut buyer_sess = self.sessions.session_for(&parties.buyer_peer).await;
        buyer_sess.active_order_id = Some(order_id);
        buyer_sess.state = OrderFlowState::AwaitingConfirmation {
            order_id,
            amount: parties.amount.clone(),
            description: String::new(),
        };
        buyer_sess.touch();
        self.sessions.update(&parties.buyer_peer, buyer_sess).await;

        let mut seller_sess = self.sessions.session_for(&parties.seller_peer).await;
        seller_sess.reset_flow();
        seller_sess.active_order_id = Some(order_id);
        self.sessions.update(&parties.seller_peer, seller_sess).await;

        tracing::info!(
            %order_id,
            seller = %mask_whatsapp_peer(&parties.seller_peer),
            buyer = %mask_whatsapp_peer(&parties.buyer_peer),
            "bank payment: WhatsApp notify sent to both parties"
        );
        Ok(())
    }

    /// Comprador confirmou recebimento: libera custódia + dispara off-ramp ao vendedor.
    async fn handle_buyer_confirm_receipt(
        &self,
        buyer_peer: &str,
        order_id: Uuid,
        amount: &str,
        mut session: UserSession,
    ) {
        let buyer_id = session.user_id;
        let idempotency_key = format!("confirm_receipt:{order_id}:{buyer_id}");

        // Gera JWT do comprador para autorizar o release.
        let bearer = self.jwt_for_user(buyer_id, apicash_auth::Role::Buyer)
            .ok()
            .map(|t| t);

        // 1. Release da custódia no Core.
        match self.core.release_custody(order_id, buyer_id, &idempotency_key, bearer.as_deref()).await {
            Ok(_) => {
                tracing::info!(%order_id, %buyer_id, "custody released by buyer");
            }
            Err(e) => {
                tracing::warn!(%order_id, error = %e, "release_custody falhou");
                self.outbound.send_text(buyer_peer, "Não foi possível confirmar o recebimento agora. Tente novamente em instantes.").await;
                session.state = OrderFlowState::AwaitingConfirmation {
                    order_id,
                    amount: amount.to_string(),
                    description: String::new(),
                };
                self.sessions.update(buyer_peer, session).await;
                return;
            }
        }

        // 2. Notifica comprador.
        self.outbound
            .send_text(buyer_peer, &message_templates::buyer_receipt_confirmed(amount))
            .await;

        // 3. Obtém seller_peer e chave PIX do vendedor.
        let seller_peer = self.payment_registry.get(order_id).await
            .map(|p| p.seller_peer);

        let seller_pix = if let Some(ref sp) = seller_peer {
            let from_registry = {
                let seller_sess = self.sessions.session_for(sp).await;
                seller_sess.seller_pix_key.clone()
            };
            if let Some(k) = from_registry {
                Some(k)
            } else if let Some(pool) = &self.pg_pool {
                crate::wa_contact_store::load_pix_key(pool, sp).await
            } else {
                None
            }
        } else {
            None
        };

        // 4. Off-ramp em background (não bloqueia).
        {
            let core = self.core.clone();
            let outbound = self.outbound.clone();
            let amount_str = amount.to_string();
            let sp = seller_peer.clone();
            match seller_pix {
                Some(ref pix) => {
                    let pix_key = pix.clone();
                    let pix_for_msg = pix.clone();
                    tokio::spawn(async move {
                        match core.off_ramp_order(order_id, &pix_key).await {
                            Ok(_) => {
                                tracing::info!(%order_id, pix_key = %pix_key, "off-ramp OK");
                                if let Some(ref sp) = sp {
                                    outbound.send_text(sp, &message_templates::seller_payment_released(&amount_str, &pix_for_msg)).await;
                                }
                            }
                            Err(e) => {
                                tracing::warn!(%order_id, error = %e, "off-ramp falhou");
                                if let Some(ref sp) = sp {
                                    outbound.send_text(sp, &message_templates::seller_payment_released_no_pix(&amount_str)).await;
                                }
                            }
                        }
                    });
                }
                None => {
                    if let Some(ref sp) = seller_peer {
                        self.outbound
                            .send_text(sp, &message_templates::seller_payment_released_no_pix(amount))
                            .await;
                    }
                    tracing::warn!(%order_id, "off-ramp: chave PIX do vendedor não encontrada");
                }
            }
        }

        // 5. Reset de estado.
        session.reset_flow();
        session.active_order_id = Some(order_id);
        self.sessions.update(buyer_peer, session).await;
        if let Some(sp) = seller_peer {
            let mut seller_s = self.sessions.session_for(&sp).await;
            seller_s.reset_flow();
            seller_s.active_order_id = Some(order_id);
            self.sessions.update(&sp, seller_s).await;
        }
    }

    /// Faz download de mídia da Cloud API e upload para MinIO (bucket disputes/).
    /// Retorna `(minio_url, sha256, evidence_kind_str, ext)` ou `None` se falhar.
    async fn upload_dispute_media(
        &self,
        dispute_id: uuid::Uuid,
        media: &crate::models::CloudMediaRef,
    ) -> Option<(String, String, String, String)> {
        let bytes = match download_cloud_media(&media.media_id).await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(media_id = %media.media_id, error = %e, "dispute: media download failed");
                return None;
            }
        };

        let ext = media.mime_type.as_deref()
            .and_then(|m| m.split('/').last())
            .map(|e| e.trim_end_matches(';').split(';').next().unwrap_or(e).to_string())
            .unwrap_or_else(|| media.kind.default_ext().to_string());

        let store = apicash_disputes::image_store::DisputeImageStore::from_env()?;
        match store.upload(dispute_id, &ext, &bytes).await {
            Ok((_key, url, sha256)) => {
                let kind_str = media.kind.to_evidence_kind().to_string();
                Some((url, sha256, kind_str, ext))
            }
            Err(e) => {
                tracing::warn!(%dispute_id, error = %e, "dispute: MinIO upload failed");
                None
            }
        }
    }

    /// Notifica comprador e vendedor quando a disputa é resolvida.
    /// Chamado via `POST /internal/dispute-resolved` pelo apicash-core.
    pub async fn notify_dispute_result(
        &self,
        order_id: uuid::Uuid,
        verdict: &str,
        amount: &str,
    ) {
        let Some(parties) = self.payment_registry.get(order_id).await else {
            tracing::warn!(%order_id, "dispute_result: parties not found in registry");
            return;
        };

        let (buyer_msg, seller_msg) = match verdict {
            "favor_buyer" | "refund_buyer" => (
                message_templates::dispute_resolved_buyer(amount),
                message_templates::dispute_resolved_seller_loss(amount),
            ),
            "favor_seller" | "release_to_seller" => (
                message_templates::dispute_resolved_buyer_loss(amount),
                message_templates::dispute_resolved_seller(amount),
            ),
            _ => (
                format!("✅ Disputa encerrada. O valor de R$ {amount} foi processado conforme a decisão."),
                format!("✅ Disputa encerrada. O valor de R$ {amount} foi processado conforme a decisão."),
            ),
        };

        self.outbound.send_text(&parties.buyer_peer, &buyer_msg).await;
        self.outbound.send_text(&parties.seller_peer, &seller_msg).await;

        // Reset estados de disputa de ambos.
        for p in [&parties.buyer_peer, &parties.seller_peer] {
            let mut s = self.sessions.session_for(p).await;
            if matches!(s.state,
                OrderFlowState::DisputeAwaitingDecision { .. }
                | OrderFlowState::DisputeSellerResponding { .. }
                | OrderFlowState::DisputeCollectingEvidence { .. }
            ) {
                s.reset_flow();
                s.active_order_id = Some(order_id);
                self.sessions.update(p, s).await;
            }
        }

        tracing::info!(%order_id, verdict, "dispute_result: both parties notified");
    }

    /// Notifica o vendedor que uma disputa foi aberta, com prazo de 72h para responder.
    async fn notify_seller_dispute_opened(&self, order_id: uuid::Uuid, buyer_session: &UserSession) {
        let Some(parties) = self.payment_registry.get(order_id).await else {
            tracing::warn!(%order_id, "dispute: seller_peer not found in payment_registry");
            return;
        };
        self.outbound
            .send_text(&parties.seller_peer, &message_templates::dispute_seller_notify(&parties.amount, 72))
            .await;
        tracing::info!(
            %order_id,
            seller_peer = %parties.seller_peer,
            buyer_user  = %buyer_session.user_id,
            "dispute: seller notified"
        );
    }

    /// Busca buyer_peer e seller_peer no DB pelo tracking_code (fallback quando o payload não traz telefones).
    async fn lookup_peers_by_tracking_code(&self, tracking_code: &str) -> (Option<String>, Option<String>) {
        let Some(pool) = &self.pg_pool else { return (None, None) };
        let row = sqlx::query(
            "SELECT buyer_peer, seller_peer FROM order_tracking_status WHERE tracking_code = $1 LIMIT 1",
        )
        .bind(tracking_code)
        .fetch_optional(pool.as_ref())
        .await;
        match row {
            Ok(Some(r)) => {
                let buyer: String = r.try_get("buyer_peer").unwrap_or_else(|_| String::new());
                let seller: String = r.try_get("seller_peer").unwrap_or_else(|_| String::new());
                (
                    if buyer.is_empty() { None } else { Some(buyer) },
                    if seller.is_empty() { None } else { Some(seller) },
                )
            }
            _ => (None, None),
        }
    }

    /// Notifica comprador (toda etapa) e vendedor (status críticos).
    pub async fn notify_tracking_step(
        &self,
        seller_phone: &str,
        buyer_phone: Option<&str>,
        order_id: Option<&str>,
        tracking_code: &str,
        step_label: &str,
        description: &str,
    ) {
        // Se buyer_phone não veio no payload, busca no DB pelo tracking_code.
        let (db_buyer, db_seller) = if buyer_phone.map(str::trim).filter(|s| !s.is_empty()).is_none() {
            self.lookup_peers_by_tracking_code(tracking_code).await
        } else {
            (None, None)
        };
        let resolved_buyer = buyer_phone.map(str::trim).filter(|s| !s.is_empty())
            .map(str::to_string)
            .or(db_buyer);
        let resolved_seller = {
            let sp = seller_phone.trim();
            if sp.is_empty() { db_seller } else { Some(sp.to_string()) }
        };

        let mut msg = String::from("📦 *Atualização de rastreio*");
        if let Some(oid) = order_id.filter(|s| !s.trim().is_empty()) {
            msg.push_str(&format!("\nPedido: `{oid}`"));
        }
        msg.push_str(&format!(
            "\n\nCódigo: `{tracking_code}`\nEtapa: *{step_label}*\n{description}"
        ));

        // Comprador recebe aviso em cada etapa.
        if let Some(bp) = resolved_buyer.as_deref() {
            self.outbound.send_text(bp, &msg).await;
            tracing::info!(
                peer = %mask_whatsapp_peer(bp),
                code = %tracking_code,
                step = %step_label,
                "tracking step: WhatsApp enviado ao comprador"
            );
        }

        // Vendedor recebe aviso nos status críticos: entrega, retorno, devolução e problema.
        let label_lc = step_label.to_lowercase();
        let is_critical = label_lc.contains("entregue")
            || label_lc.contains("delivered")
            || label_lc.contains("retorno")
            || label_lc.contains("return")
            || label_lc.contains("devolvido")
            || label_lc.contains("returned")
            || label_lc.contains("problema")
            || label_lc.contains("exception");
        if is_critical {
            if let Some(sp) = resolved_seller.as_deref() {
                let seller_msg = message_templates::tracking_critical_seller_notify(
                    tracking_code,
                    step_label,
                    order_id,
                );
                self.outbound.send_text(sp, &seller_msg).await;
                tracing::info!(
                    peer = %mask_whatsapp_peer(sp),
                    code = %tracking_code,
                    step = %step_label,
                    "tracking step: WhatsApp enviado ao vendedor (status crítico)"
                );
            }
        }
    }

    /// PN confirmado pelo WhatsApp (evita divergência 554198… vs 554188… do cartão).
    async fn canonical_buyer_peer_for_whatsapp(&self, raw: &str) -> String {
        if let Outbound::Rust { client, .. } = &*self.outbound {
            if let Some(d) = canonical_whatsapp_peer_digits(client.as_ref(), raw).await {
                return d;
            }
        }
        holdfy::canonical_peer_key(raw).unwrap_or_else(|| {
            raw.chars().filter(|c| c.is_ascii_digit()).collect()
        })
    }

    async fn sync_buyer_sessions_for_proposal(
        &self,
        buyer_peer_key: &str,
        seller_peer: &str,
        amt: &str,
        description: &str,
    ) {
        let state = OrderFlowState::BuyerPendingSellerProposal {
            seller_peer_key: seller_peer.to_string(),
            amount: amt.to_string(),
            description: description.to_string(),
        };
        for alt in peer_lookup_digit_variants(buyer_peer_key) {
            let mut sess = self.sessions.session_for(&alt).await;
            sess.state = state.clone();
            sess.touch();
            self.sessions.update(&alt, sess).await;
        }
    }

    fn jwt_for_user(&self, user_id: Uuid, role: Role) -> Result<String, CoreApiError> {
        self.jwt
            .generate_token(user_id, role, None)
            .map_err(|e| CoreApiError::Api {
                status: 500,
                body: format!("JWT signer misconfigured: {e}"),
            })
    }

    fn merge_holdfy_draft(draft: &mut OrderDraft, body: &str, ev: &WhatsAppEvent) {
        let (amt, phone) = parse_loose_fields(body, Some(ev));
        if let Some(a) = amt {
            draft.amount = Some(a);
        }
        if let Some(p) = phone {
            draft.counterparty_peer_key =
                holdfy::canonical_peer_key(&p).or(Some(p));
        }
    }

    fn invalid_phone_attempt(body: &str, ev: &WhatsAppEvent) -> bool {
        if parse_loose_fields(body, Some(ev)).1.is_some() {
            return false;
        }
        if holdfy::contact_phone_rejected(ev) {
            return true;
        }
        let d = holdfy::phone::digits_only(body);
        d.len() >= 8 && holdfy::normalize_br_mobile(body).is_none()
    }

    fn holdfy_creating_step(collect: HoldfyCollectStep) -> CreatingOrderStep {
        match collect {
            HoldfyCollectStep::AskAmount => CreatingOrderStep::AskAmount,
            HoldfyCollectStep::AskPhone => CreatingOrderStep::AskCounterparty,
            HoldfyCollectStep::Ready => CreatingOrderStep::WaitingBuyerAccept,
        }
    }

    async fn prompt_holdfy_collect(&self, peer: &str, step: HoldfyCollectStep) {
        let msg = match step {
            HoldfyCollectStep::AskAmount => message_templates::ask_holdfy_amount(),
            HoldfyCollectStep::AskPhone => message_templates::ask_holdfy_phone(),
            HoldfyCollectStep::Ready => return,
        };
        self.outbound.send_text(peer, msg).await;
    }

    /// Envia proposta ao comprador quando valor e celular já estão no rascunho.
    async fn try_send_holdfy_proposal(
        &self,
        session: &mut UserSession,
        seller_peer: &str,
        draft: OrderDraft,
    ) -> Result<bool, CoreApiError> {
        let Some(amt) = draft.amount.clone() else {
            return Ok(false);
        };
        let Some(raw_buyer) = draft.counterparty_peer_key.clone() else {
            return Ok(false);
        };
        let buyer_peer_key = self.canonical_buyer_peer_for_whatsapp(&raw_buyer).await;
        if order_flow::peers_same_phone(&buyer_peer_key, seller_peer) {
            session.reset_flow();
            self.sessions.update(seller_peer, session.clone()).await;
            self.outbound
                .send_text(seller_peer, message_templates::counterparty_same_as_sender())
                .await;
            return Ok(true);
        }
        let description = draft.description.clone().unwrap_or_else(|| {
            format!("HoldFy (contato comprador ~{})", mask_whatsapp_peer(&buyer_peer_key))
        });

        if draft.seller_document.is_none() {
            session.state = OrderFlowState::CreatingOrder {
                step: CreatingOrderStep::AskSellerDocument,
                draft: OrderDraft {
                    counterparty_peer_key: Some(buyer_peer_key.clone()),
                    amount: Some(amt.clone()),
                    description: Some(description.clone()),
                    seller_document: None,
                    listing_id: draft.listing_id,
                    listing_photos: draft.listing_photos.clone(),
                    listing_source_url: draft.listing_source_url.clone(),
                    listing_price_suggested: draft.listing_price_suggested.clone(),
                },
            };
            session.touch();
            self.sessions.update(seller_peer, session.clone()).await;
            self.outbound
                .send_text(seller_peer, message_templates::ask_seller_document())
                .await;
            return Ok(true);
        }

        session.state = OrderFlowState::CreatingOrder {
            step: CreatingOrderStep::WaitingBuyerAccept,
            draft: OrderDraft {
                counterparty_peer_key: Some(buyer_peer_key.clone()),
                amount: Some(amt.clone()),
                description: Some(description.clone()),
                seller_document: draft.seller_document.clone(),
                listing_id: draft.listing_id,
                listing_photos: draft.listing_photos.clone(),
                listing_source_url: draft.listing_source_url.clone(),
                listing_price_suggested: draft.listing_price_suggested.clone(),
            },
        };
        session.touch();
        self.sessions.update(seller_peer, session.clone()).await;

        self.sync_buyer_sessions_for_proposal(
            &buyer_peer_key,
            seller_peer,
            &amt,
            &description,
        )
        .await;

        let masked_seller = mask_whatsapp_peer(seller_peer);
        let masked_buyer = mask_whatsapp_peer(&buyer_peer_key);
        let seller_doc = draft.seller_document.as_deref().unwrap_or("");
        let seller_name = session.contact_name.as_deref();
        let listing_url = draft.listing_source_url.as_deref();
        let listing_price = draft.listing_price_suggested.as_deref();

        let buyer_ok = self
            .outbound
            .send_text(
                &buyer_peer_key,
                message_templates::buyer_proposal_before_accept(
                    &masked_seller, &amt, &description, seller_name, seller_doc, listing_url, listing_price,
                ),
            )
            .await;

        if !buyer_ok {
            session.reset_flow();
            self.sessions.update(seller_peer, session.clone()).await;
            let mut buyer_sess = self.sessions.session_for(&buyer_peer_key).await;
            buyer_sess.reset_flow();
            self.sessions.update(&buyer_peer_key, buyer_sess).await;
            self.outbound
                .send_text(
                    seller_peer,
                    message_templates::buyer_whatsapp_unreachable(&masked_buyer),
                )
                .await;
            return Ok(true);
        }

        self.outbound
            .send_text(
                seller_peer,
                message_templates::seller_waiting_buyer_accept(&amt, &masked_buyer),
            )
            .await;
        Ok(true)
    }

    /// Vendedor enviou um link de produto — importar via core API, salvar no banco e pré-preencher rascunho.
    async fn start_holdfy_flow(
        &self,
        session: &mut UserSession,
        seller_peer: &str,
        body: &str,
        ev: &WhatsAppEvent,
    ) -> Result<(), CoreApiError> {
        // Preserva amount/phone se vieram na frase inicial (ex.: "holdfy 200 para 41999...")
        let mut draft = OrderDraft::default();
        if let Some(parsed) = parse_holdfy_message(body, Some(ev)) {
            draft.amount = parsed.amount;
            draft.counterparty_peer_key = parsed.phone;
        } else {
            Self::merge_holdfy_draft(&mut draft, body, ev);
        }

        // Primeiro passo: pedir o link do anúncio
        session.active_order_id = None;
        session.state = OrderFlowState::AwaitingListingUrl { draft };
        session.touch();
        self.sessions.update(seller_peer, session.clone()).await;
        self.outbound.send_text(seller_peer, message_templates::ask_listing_url()).await;
        Ok(())
    }

    /// Trata a resposta ao pedido de link do anúncio.
    /// - URL → importa e avança
    /// - "pular" → segue o fluxo sem anúncio
    async fn handle_listing_url_response(
        &self,
        session: &mut UserSession,
        seller_peer: &str,
        body: &str,
        _ev: &WhatsAppEvent,
        draft: OrderDraft,
    ) -> Result<(), CoreApiError> {
        let skip = matches!(body.to_lowercase().trim(), "pular" | "skip" | "não" | "nao" | "sem anuncio" | "sem anúncio");

        if skip {
            // Continua sem anúncio
            self.advance_to_collect(session, seller_peer, draft).await
        } else if order_flow::is_product_url(body) {
            // Importa o link e avança
            let mut draft = draft;
            self.outbound.send_text(seller_peer, message_templates::importing_product()).await;
            match self.core.import_listing(body, session.user_id).await {
                Ok(resp) => {
                    let title = resp.title.clone();
                    let price_str = resp.price_suggested.clone();
                    draft.description = Some(title.clone());
                    draft.listing_id = resp.listing_id;
                    draft.listing_photos = resp.photos.clone();
                    draft.listing_source_url = Some(resp.source_url.clone());
                    draft.listing_price_suggested = resp.price_suggested.clone();
                    // Preenche amount com preço do anúncio se ainda não foi informado
                    if draft.amount.is_none() {
                        draft.amount = price_str.clone();
                    }
                    // Envia vídeo se disponível, senão imagem — confirmação ao vendedor
                    if let Some(video_url) = &resp.video_url {
                        match download_url_bytes(video_url).await {
                            Ok(bytes) => {
                                self.outbound.send_video_bytes(seller_peer, &bytes, None).await;
                            }
                            Err(_) => {
                                if let Some(photo_url) = resp.photos.first() {
                                    if let Ok(bytes) = download_url_bytes(photo_url).await {
                                        self.outbound.send_image_bytes(seller_peer, &bytes, None).await;
                                    }
                                }
                            }
                        }
                    } else if let Some(photo_url) = resp.photos.first() {
                        if let Ok(bytes) = download_url_bytes(photo_url).await {
                            self.outbound.send_image_bytes(seller_peer, &bytes, None).await;
                        }
                    }
                    let msg = match &price_str {
                        Some(p) => message_templates::product_imported_with_price(&title, p, &resp.source_url),
                        None => message_templates::product_imported_no_price(&title, &resp.source_url),
                    };
                    self.outbound.send_text(seller_peer, &msg).await;
                    self.advance_to_collect(session, seller_peer, draft).await
                }
                Err(e) => {
                    tracing::warn!(peer = %mask_whatsapp_peer(seller_peer), error = %e, "import listing failed");
                    self.outbound.send_text(seller_peer, message_templates::product_import_failed()).await;
                    // Dá outra hipótese de enviar link ou pular
                    session.state = OrderFlowState::AwaitingListingUrl { draft };
                    session.touch();
                    self.sessions.update(seller_peer, session.clone()).await;
                    Ok(())
                }
            }
        } else {
            // Resposta não reconhecida — pede de novo
            self.outbound.send_text(seller_peer, message_templates::ask_listing_url()).await;
            session.state = OrderFlowState::AwaitingListingUrl { draft };
            session.touch();
            self.sessions.update(seller_peer, session.clone()).await;
            Ok(())
        }
    }

    async fn advance_to_collect(
        &self,
        session: &mut UserSession,
        seller_peer: &str,
        draft: OrderDraft,
    ) -> Result<(), CoreApiError> {
        let collect = next_collect_step(
            draft.amount.as_deref(),
            draft.counterparty_peer_key.as_deref(),
        );
        if collect == HoldfyCollectStep::Ready {
            return self.try_send_holdfy_proposal(session, seller_peer, draft).await.map(|_| ());
        }
        session.state = OrderFlowState::CreatingOrder {
            step: Self::holdfy_creating_step(collect),
            draft: draft.clone(),
        };
        session.touch();
        self.sessions.update(seller_peer, session.clone()).await;
        self.prompt_holdfy_collect(seller_peer, collect).await;
        Ok(())
    }

    async fn advance_holdfy_collecting(
        &self,
        session: &mut UserSession,
        seller_peer: &str,
        step: CreatingOrderStep,
        mut draft: OrderDraft,
        body: &str,
        ev: &WhatsAppEvent,
    ) -> Result<(), CoreApiError> {
        Self::merge_holdfy_draft(&mut draft, body, ev);

        if matches!(step, CreatingOrderStep::AskCounterparty)
            && draft.counterparty_peer_key.is_none()
            && Self::invalid_phone_attempt(body, ev)
        {
            session.state = OrderFlowState::CreatingOrder { step, draft };
            self.sessions.update(seller_peer, session.clone()).await;
            self.outbound
                .send_text(seller_peer, message_templates::invalid_holdfy_phone())
                .await;
            return Ok(());
        }

        let collect = next_collect_step(
            draft.amount.as_deref(),
            draft.counterparty_peer_key.as_deref(),
        );
        if collect == HoldfyCollectStep::Ready {
            return self
                .try_send_holdfy_proposal(session, seller_peer, draft)
                .await
                .map(|_| ());
        }

        if matches!(step, CreatingOrderStep::AskAmount) && draft.amount.is_none() {
            session.state = OrderFlowState::CreatingOrder { step, draft };
            self.sessions.update(seller_peer, session.clone()).await;
            self.outbound
                .send_text(seller_peer, message_templates::invalid_amount())
                .await;
            return Ok(());
        }

        session.state = OrderFlowState::CreatingOrder {
            step: Self::holdfy_creating_step(collect),
            draft: draft.clone(),
        };
        session.touch();
        self.sessions.update(seller_peer, session.clone()).await;
        self.prompt_holdfy_collect(seller_peer, collect).await;
        Ok(())
    }

    /// Salva contato no banco (fire-and-forget) e atualiza sessão em memória.
    /// `authoritative`: se true (NFS-e / Receita Federal), sobrescreve sempre o nome existente.
    /// Se false (push_name do WhatsApp), só escreve se ainda não houver nome.
    async fn save_peer_contact_inner(&self, peer: &str, document: &str, name: Option<String>, situation: Option<String>, authoritative: bool) {
        let user_id = crate::session::user_id_for_peer_key(peer);

        let mut session = self.sessions.session_for(peer).await;
        if authoritative {
            if name.is_some() {
                session.contact_name = name.clone();
            }
        } else if session.contact_name.is_none() {
            session.contact_name = name.clone();
        }
        if session.document.is_none() && !document.is_empty() {
            session.document = Some(document.to_string());
        }
        session.contact_loaded = true;
        self.sessions.update(peer, session).await;

        // Persiste no Postgres em background
        if let Some(pool) = &self.pg_pool {
            let doc = if document.is_empty() { None } else { Some(document.to_string()) };
            let contact = crate::wa_contact_store::WaContact {
                peer_key: peer.to_string(),
                user_id,
                name,
                document: doc,
                document_type: None,
                situation,
                pix_key: None,
            };
            let pool = pool.clone();
            tokio::spawn(async move {
                crate::wa_contact_store::save_contact(&pool, &contact).await;
            });
        }
    }

    /// Consulta NFS-e com cache em banco. Evita chamar o portal da Receita para o mesmo CPF/CNPJ.
    async fn lookup_person_cached(
        &self,
        document: &str,
    ) -> (Option<crate::nfse_client::PersonLookup>, crate::nfse_client::LookupStatus) {
        let digits: String = document.chars().filter(|c| c.is_ascii_digit()).collect();

        if let Some(pool) = &self.pg_pool {
            if let Some(cached) = crate::wa_contact_store::get_nfse_cache(pool, &digits).await {
                tracing::info!(doc_len = digits.len(), name = cached.name.as_deref().unwrap_or("?"), "nfse_cache: hit — Receita não consultada");
                return (Some(cached), crate::nfse_client::LookupStatus::Ok);
            }
        }

        let (person, status) = crate::nfse_client::lookup_person(document).await;

        if let (Some(ref p), crate::nfse_client::LookupStatus::Ok) = (&person, &status) {
            if p.name.is_some() {
                if let Some(pool) = &self.pg_pool {
                    let pool = pool.clone();
                    let p = p.clone();
                    tokio::spawn(async move {
                        crate::wa_contact_store::save_nfse_cache(&pool, &p).await;
                    });
                }
            }
        }

        (person, status)
    }

    /// Nome da Receita Federal — sobrescreve push_name (fonte autoritativa).
    async fn save_peer_contact(&self, peer: &str, document: &str, name: Option<String>, situation: Option<String>) {
        self.save_peer_contact_inner(peer, document, name, situation, true).await;
    }

    /// Nome do perfil WhatsApp — só salva se não houver nome ainda.
    async fn save_peer_push_name(&self, peer: &str, name: String) {
        self.save_peer_contact_inner(peer, "", Some(name), None, false).await;
    }

    /// Carrega contato do banco na primeira mensagem da sessão (lazy).
    async fn maybe_load_contact(&self, session: &mut UserSession) {
        if session.contact_loaded {
            return;
        }
        session.contact_loaded = true;
        if let Some(pool) = &self.pg_pool {
            if let Some(c) = crate::wa_contact_store::load_contact(pool, &session.peer_id).await {
                session.contact_name = c.name;
                session.document = c.document;
            }
        }
    }

    /// Primeiro nome do contacto, para saudações curtas.
    fn first_name(full_name: &str) -> &str {
        full_name.split_whitespace().next().unwrap_or(full_name)
    }

    async fn seller_document_from_draft(&self, seller_peer_key: &str) -> Option<String> {
        let seller_sess = self.sessions.session_for(seller_peer_key).await;
        match &seller_sess.state {
            OrderFlowState::CreatingOrder { draft, .. } => draft.seller_document.clone(),
            _ => None,
        }
    }

    /// Comprador aceitou: pede CPF/CNPJ dele se o vendedor já tiver informado o dele.
    async fn begin_buyer_document_collection(
        &self,
        session: &mut UserSession,
        buyer_peer: &str,
        seller_peer_key: String,
        amount: String,
        description: String,
    ) -> Result<(), CoreApiError> {
        let Some(seller_document) = self.seller_document_from_draft(&seller_peer_key).await else {
            session.touch();
            self.sessions.update(buyer_peer, session.clone()).await;
            self.outbound
                .send_text(
                    buyer_peer,
                    message_templates::seller_document_pending_before_buyer(),
                )
                .await;
            let mut seller_sess = self.sessions.session_for(&seller_peer_key).await;
            if let OrderFlowState::CreatingOrder { draft, .. } = &seller_sess.state {
                if draft.seller_document.is_none() {
                    seller_sess.state = OrderFlowState::CreatingOrder {
                        step: CreatingOrderStep::AskSellerDocument,
                        draft: draft.clone(),
                    };
                    seller_sess.touch();
                    self.sessions
                        .update(&seller_peer_key, seller_sess)
                        .await;
                    self.outbound
                        .send_text(
                            &seller_peer_key,
                            message_templates::ask_seller_document(),
                        )
                        .await;
                }
            }
            return Ok(());
        };

        session.state = OrderFlowState::AwaitingBuyerDocument {
            seller_peer_key,
            seller_document,
            amount,
            description,
        };
        session.touch();
        self.sessions.update(buyer_peer, session.clone()).await;
        self.outbound
            .send_text(buyer_peer, message_templates::ask_buyer_document())
            .await;
        Ok(())
    }

    /// Vendedor informou CPF/CNPJ; consulta Receita e envia proposta ao comprador.
    async fn accept_seller_document_and_propose(
        &self,
        session: &mut UserSession,
        seller_peer: &str,
        mut draft: OrderDraft,
        document: &str,
    ) -> Result<(), CoreApiError> {
        draft.seller_document = Some(document.to_string());
        let (seller_person, seller_lookup_status) =
            self.lookup_person_cached(document).await;
        let seller_name = seller_person.as_ref().and_then(|p| p.name.clone());
        let seller_situation = seller_person.as_ref().and_then(|p| p.situation.clone());

        // NFS-e como fonte primária; push_name da sessão como fallback
        let display_name = seller_name.as_deref().or(session.contact_name.as_deref());
        if display_name.is_none() {
            tracing::warn!(
                peer = %mask_whatsapp_peer(seller_peer),
                status = ?seller_lookup_status,
                "nfse: nome não obtido para CPF/CNPJ do vendedor e push_name ausente"
            );
        }

        // Sempre confirmar o documento ao vendedor (obrigatório)
        // NFS-e separado do push_name para que o label seja correto
        let push_name_fallback = session.contact_name.as_deref().filter(|_| seller_name.is_none());
        self.outbound
            .send_text(
                seller_peer,
                &message_templates::document_confirmed(
                    document,
                    seller_name.as_deref(),
                    seller_situation.as_deref(),
                    push_name_fallback,
                ),
            )
            .await;

        // NFS-e tem precedência — sobrescreve sempre o push_name
        if seller_name.is_some() {
            session.contact_name = seller_name.clone();
        }
        session.document = Some(document.to_string());
        session.contact_loaded = true;

        // Persiste em background
        self.save_peer_contact(seller_peer, document, seller_name, seller_situation).await;

        if let Err(e) = self
            .core
            .calculate_risk_score_internal_only(
                session.user_id,
                document,
                &[],
            )
            .await
        {
            tracing::warn!(
                peer = %mask_whatsapp_peer(seller_peer),
                user_id = %session.user_id,
                error = %e,
                "core_api: seller risk score failed (continuing)"
            );
        }

        // Pede a chave PIX antes de enviar a proposta.
        self.ask_or_confirm_seller_pix(session, seller_peer, draft).await
    }

    /// Depois do CPF confirmado: verifica se já há chave PIX e pede/confirma.
    async fn ask_or_confirm_seller_pix(
        &self,
        session: &mut UserSession,
        seller_peer: &str,
        draft: OrderDraft,
    ) -> Result<(), CoreApiError> {
        // Tenta carregar chave PIX já guardada (DB ou sessão).
        let stored_pix = if let Some(k) = &session.seller_pix_key {
            Some(k.clone())
        } else if let Some(pool) = &self.pg_pool {
            crate::wa_contact_store::load_pix_key(pool, seller_peer).await
        } else {
            None
        };

        if let Some(ref pix) = stored_pix {
            session.seller_pix_key = Some(pix.clone());
            session.state = OrderFlowState::CreatingOrder {
                step: CreatingOrderStep::AskSellerPix,
                draft,
            };
            session.touch();
            self.sessions.update(seller_peer, session.clone()).await;
            self.outbound
                .send_text(seller_peer, &message_templates::seller_pix_already_stored(pix))
                .await;
        } else {
            session.state = OrderFlowState::CreatingOrder {
                step: CreatingOrderStep::AskSellerPix,
                draft,
            };
            session.touch();
            self.sessions.update(seller_peer, session.clone()).await;
            self.outbound
                .send_text(seller_peer, message_templates::ask_seller_pix_key())
                .await;
        }
        Ok(())
    }

    /// Após comprador responder *ACEITO* e fornecer documento: criar pedido, enviar PIX a B e avisos a A/B.
    async fn finalize_order_after_buyer_accepted(
        &self,
        seller_peer_key: &str,
        buyer_peer_key: &str,
        amount: String,
        description: String,
        seller_document: &str,
        buyer_document: &str,
        peer_hint_buyer: &str,
    ) -> Result<(), CoreApiError> {
        let buyer_id = crate::session::user_id_for_peer_key(buyer_peer_key);
        let seller_id = crate::session::user_id_for_peer_key(seller_peer_key);
        let seller_bearer = self.jwt_for_user(seller_id, Role::Seller)?;
        let links: Vec<String> = Vec::new();

        let mut seller_sess = self.sessions.session_for(seller_peer_key).await;
        match &seller_sess.state {
            OrderFlowState::CreatingOrder {
                step: CreatingOrderStep::WaitingBuyerAccept,
                draft,
            } if draft
                .counterparty_peer_key
                .as_ref()
                .is_some_and(|bpk| order_flow::peers_same_phone(bpk, buyer_peer_key)) => {}
            _ => {
                self.outbound
                    .send_text(
                        buyer_peer_key,
                        "Este convite não vale mais. Peça novo pedido ao vendedor.",
                    )
                    .await;
                let mut buyer_sess_local = self.sessions.session_for(buyer_peer_key).await;
                buyer_sess_local.reset_flow();
                self.sessions.update(buyer_peer_key, buyer_sess_local).await;
                return Ok(());
            }
        }

        let social_empty: &[String] = links.as_slice();

        if let Err(e) = self
            .core
            .calculate_risk_score_internal_only(buyer_id, buyer_document, social_empty)
            .await
        {
            tracing::warn!(
                peer = %peer_hint_buyer,
                buyer_id = %buyer_id,
                error = %e,
                "core_api: calculate_risk_score_internal_only failed (continuing). Configure APICASH_API_KEY for antifraude."
            );
        }
        if let Err(e) = self
            .core
            .calculate_risk_score_internal_only(seller_id, seller_document, social_empty)
            .await
        {
            tracing::warn!(
                seller_id = %seller_id,
                error = %e,
                "core_api: seller risk score failed (continuing)"
            );
        }

        let (buyer_person, buyer_lookup_status) =
            self.lookup_person_cached(buyer_document).await;
        let buyer_name: Option<String> = buyer_person.as_ref().and_then(|p| p.name.clone());
        let buyer_situation = buyer_person.as_ref().and_then(|p| p.situation.clone());

        // NFS-e como fonte primária; carregar sessão do comprador para push_name como fallback
        let buyer_session = self.sessions.session_for(buyer_peer_key).await;
        let buyer_display_name = buyer_name.as_deref().or(buyer_session.contact_name.as_deref());
        if buyer_display_name.is_none() {
            tracing::warn!(
                peer = %mask_whatsapp_peer(buyer_peer_key),
                status = ?buyer_lookup_status,
                "nfse: nome não obtido para CPF/CNPJ do comprador e push_name ausente"
            );
        }

        // Sempre confirmar o documento ao comprador (obrigatório)
        let buyer_push_fallback = buyer_session.contact_name.as_deref().filter(|_| buyer_name.is_none());
        self.outbound
            .send_text(
                buyer_peer_key,
                &message_templates::document_confirmed(
                    buyer_document,
                    buyer_name.as_deref(),
                    buyer_situation.as_deref(),
                    buyer_push_fallback,
                ),
            )
            .await;

        // Persiste contato do comprador
        self.save_peer_contact(buyer_peer_key, buyer_document, buyer_name.clone(), buyer_situation).await;

        let order = match self
            .core
            .create_order(
                buyer_id,
                seller_id,
                &amount,
                buyer_document,
                social_empty,
                Some(description.as_str()),
                buyer_name.as_deref(),
                Some(&seller_bearer),
            )
            .await
        {
            Ok(o) => o,
            Err(CoreApiError::Api {
                status: s @ 403,
                body,
                ..
            }) if body.contains("on-ramp blocked")
                || body.contains("blocked by anti-fraud policy") =>
            {
                tracing::warn!(
                    peer = %peer_hint_buyer,
                    user_id = %buyer_id,
                    status = s,
                    body = %body,
                    "core_api: create_order blocked"
                );
                seller_sess.reset_flow();
                self.sessions.update(seller_peer_key, seller_sess).await;
                let mut buyer_sess = self.sessions.session_for(buyer_peer_key).await;
                buyer_sess.reset_flow();
                self.sessions.update(buyer_peer_key, buyer_sess).await;
                self.outbound
                    .send_text(buyer_peer_key, message_templates::on_ramp_blocked())
                    .await;
                return Ok(());
            }
            Err(e) => {
                tracing::warn!(
                    peer = %peer_hint_buyer,
                    user_id = %buyer_id,
                    error = %e,
                    "core_api: create_order failed"
                );
                seller_sess.reset_flow();
                self.sessions.update(seller_peer_key, seller_sess).await;
                let mut buyer_sess = self.sessions.session_for(buyer_peer_key).await;
                buyer_sess.reset_flow();
                self.sessions.update(buyer_peer_key, buyer_sess).await;
                self.outbound
                    .send_text(
                        buyer_peer_key,
                        "Não deu para criar o pedido agora. Tente de novo já.",
                    )
                    .await;
                self.outbound
                    .send_text(
                        seller_peer_key,
                        "Falhou ao criar o pedido. *Novo pedido* quando puder.",
                    )
                    .await;
                return Ok(());
            }
        };

        let order_id = order.id;

        // Vincula o listing ao pedido se o vendedor tiver importado um anúncio
        {
            let seller_draft_listing = match &seller_sess.state {
                OrderFlowState::CreatingOrder { draft, .. } => draft.listing_id,
                _ => None,
            };
            if let Some(lid) = seller_draft_listing {
                let core = self.core.clone();
                tokio::spawn(async move {
                    let _ = core.link_listing_to_order(lid, order_id).await;
                });
            }
        }

        self.payment_registry
            .register(
                order_id,
                OrderPaymentParties {
                    seller_peer: seller_peer_key.to_string(),
                    buyer_peer: buyer_peer_key.to_string(),
                    amount: amount.clone(),
                },
            )
            .await;

        tracing::info!(
            order_id = %order_id,
            seller_id = %seller_id,
            buyer_id = %buyer_id,
            buyer_peer = %mask_whatsapp_peer(buyer_peer_key),
            "tri_party: pedido criado após ACEITO do comprador"
        );

        // Gerar resumo da conversa no momento da criação do pedido
        self.trigger_summary(buyer_peer_key, buyer_id, Some(order_id), SummaryTrigger::OrderCreated);
        self.trigger_summary(seller_peer_key, seller_id, Some(order_id), SummaryTrigger::OrderCreated);

        let pix_br_code = match order.pix_br_code.clone() {
            Some(v) if !v.trim().is_empty() => v,
            _ => {
                seller_sess.reset_flow();
                self.sessions.update(seller_peer_key, seller_sess).await;
                let mut buyer_sess_reset = self.sessions.session_for(buyer_peer_key).await;
                buyer_sess_reset.reset_flow();
                self.sessions.update(buyer_peer_key, buyer_sess_reset).await;
                self.outbound
                    .send_text(
                        seller_peer_key,
                        "Tem pedido, mas sem código PIX no sistema. Suporte.",
                    )
                    .await;
                self.outbound
                    .send_text(
                        buyer_peer_key,
                        "Tem pedido, mas sem código PIX no sistema. Suporte.",
                    )
                    .await;
                return Ok(());
            }
        };

        let png_ok = match qr_code::pix_qr_png_bytes(&pix_br_code) {
            Ok(b) => Some(b),
            Err(e) => {
                tracing::warn!(error = %e, "qr png failed");
                None
            }
        };

        let mut buyer_sess = self.sessions.session_for(buyer_peer_key).await;
        buyer_sess.state = OrderFlowState::AwaitingPayment {
            order_id,
            amount: amount.clone(),
            description: description.clone(),
            pix_br_code: pix_br_code.clone(),
        };
        buyer_sess.active_order_id = Some(order_id);
        buyer_sess.touch();
        self.sessions.update(buyer_peer_key, buyer_sess).await;

        self.outbound
            .send_text(
                buyer_peer_key,
                message_templates::buyer_payment_intro(&amount, &description),
            )
            .await;
        self.outbound
            .send_text(
                buyer_peer_key,
                message_templates::order_control_number_only(&order_id),
            )
            .await;
        if let Some(ref bytes) = png_ok {
            self.outbound
                .send_image_bytes(buyer_peer_key, bytes, None)
                .await;
        }
        self.outbound
            .send_text(
                buyer_peer_key,
                message_templates::pix_copy_paste_plain(&pix_br_code),
            )
            .await;

        seller_sess.active_order_id = Some(order_id);
        seller_sess.state = OrderFlowState::Idle;
        seller_sess.touch();
        self.sessions.update(seller_peer_key, seller_sess).await;

        let masked_buyer = mask_whatsapp_peer(buyer_peer_key);
        self.outbound
            .send_text(
                seller_peer_key,
                message_templates::seller_order_created_notice(
                    &amount,
                    &description,
                    &masked_buyer,
                    buyer_display_name,
                    buyer_document,
                ),
            )
            .await;
        self.outbound
            .send_text(
                seller_peer_key,
                message_templates::order_control_number_only(&order_id),
            )
            .await;
        self.outbound
            .send_text(
                seller_peer_key,
                message_templates::pix_copy_paste_plain(&pix_br_code),
            )
            .await;

        Ok(())
    }

    pub async fn handle_event(&self, ev: WhatsAppEvent) -> Result<(), CoreApiError> {
        let peer = ev.sender_id.clone();
        let peer_hint = mask_whatsapp_peer(&peer);
        let mut session = self.sessions.session_for(&peer).await;
        self.maybe_load_contact(&mut session).await;

        // Salva o nome de perfil WhatsApp quando ainda não temos nome melhor (não sobrescreve NFS-e)
        if session.contact_name.is_none() {
            if let Some(ref pn) = ev.push_name {
                self.save_peer_push_name(&peer, pn.clone()).await;
                session.contact_name = Some(pn.clone());
            }
        }

        self.sessions.update(&peer, session.clone()).await;
        let body = ev.body.trim();

        // Persistir mensagem inbound no MongoDB
        let state_tag = format!("{:?}", session.state).split(' ').next().unwrap_or("Unknown").to_string();
        self.record_inbound(&ev, &state_tag, session.active_order_id);

        tracing::info!(
            peer = %peer_hint,
            user_id = %session.user_id,
            state = ?session.state,
            body_len = ev.body.len(),
            "whatsapp: mensagem recebida"
        );

        // Global commands (work in any state).
        if order_flow::is_cancel(body) || body.eq_ignore_ascii_case("CANCELAR") {
            match &session.state {
                OrderFlowState::BuyerPendingSellerProposal {
                    seller_peer_key,
                    amount,
                    ..
                } => {
                    let spk = seller_peer_key.clone();
                    let amt = amount.clone();
                    session.reset_flow();
                    self.sessions.update(&peer, session).await;
                    let mut seller_s = self.sessions.session_for(&spk).await;
                    if matches!(
                        seller_s.state,
                        OrderFlowState::CreatingOrder {
                            step: CreatingOrderStep::WaitingBuyerAccept,
                            ..
                        }
                    ) {
                        seller_s.reset_flow();
                        self.sessions.update(&spk, seller_s).await;
                        self.outbound
                            .send_text(
                                &spk,
                                message_templates::seller_proposal_cancelled_by_buyer(&amt),
                            )
                            .await;
                    }
                    self.outbound
                        .send_text(&peer, message_templates::cancelled())
                        .await;
                    return Ok(());
                }
                _ => {
                    if let OrderFlowState::CreatingOrder {
                        step: CreatingOrderStep::WaitingBuyerAccept,
                        draft,
                    } = &session.state
                    {
                        if let Some(bpk) = draft.counterparty_peer_key.clone() {
                            let mut bs = self.sessions.session_for(&bpk).await;
                            let clear = matches!(
                                &bs.state,
                                OrderFlowState::BuyerPendingSellerProposal {
                                    seller_peer_key,
                                    ..
                                } if seller_peer_key.as_str() == peer.as_str()
                            );
                            if clear {
                                bs.reset_flow();
                                self.sessions.update(&bpk, bs).await;
                                self.outbound
                                    .send_text(
                                        &bpk,
                                        "Vendedor encerrou antes do aceite. Nada cobrado.",
                                    )
                                    .await;
                            }
                        }
                    }
                    session.reset_flow();
                    self.sessions.update(&peer, session).await;
                    self.outbound
                        .send_text(&peer, message_templates::cancelled())
                        .await;
                    return Ok(());
                }
            }
        }
        if order_flow::is_help(body) || body.eq_ignore_ascii_case("AJUDA") {
            let welcome_msg = match session.contact_name.as_deref() {
                Some(name) => message_templates::welcome_known(Self::first_name(name)),
                None => message_templates::welcome().to_string(),
            };
            self.outbound.send_text(&peer, &welcome_msg).await;
            self.outbound
                .send_text(&peer, message_templates::welcome_help())
                .await;
            return Ok(());
        }

        // ─── Vendedor: contestar disputa ──────────────────────────────────────────
        if order_flow::is_contest_dispute(&ev.body) {
            // Busca pedido ativo deste vendedor que tem disputa aberta.
            let dispute_order = self.payment_registry.find_order_for_seller(&peer).await
                .map(|(oid, _)| oid);
            if let Some(order_id) = dispute_order {
                session.state = OrderFlowState::DisputeSellerResponding {
                    order_id,
                    evidence_count: 0,
                };
                session.touch();
                self.sessions.update(&peer, session).await;
                self.outbound
                    .send_text(&peer, message_templates::dispute_collect_counter_evidence())
                    .await;
                return Ok(());
            }
        }

        // ─── Vendedor: coleta de contra-evidências ────────────────────────────────
        if let OrderFlowState::DisputeSellerResponding { order_id, evidence_count } = session.state.clone() {
            let trimmed = ev.body.trim().to_lowercase();
            let is_done = trimmed == "pronto" || trimmed == "ok" || trimmed == "enviei" || evidence_count >= 5;

            if is_done && ev.media.is_none() {
                // Contra-evidências submetidas — re-aciona análise.
                let core = self.core.clone();
                tokio::spawn(async move {
                    let _ = core.trigger_dispute_analysis(order_id).await;
                });
                session.state = OrderFlowState::Idle;
                session.touch();
                self.sessions.update(&peer, session).await;
                self.outbound.send_text(&peer, "✅ Contra-evidências registradas. A análise foi atualizada. Você será notificado com o resultado.").await;
                return Ok(());
            }

            let new_count = if let Some(ref media_ref) = ev.media {
                let disputes = self.core.get_dispute_for_order(order_id).await
                    .ok().flatten();
                if let Some(dispute_id) = disputes {
                    if let Some((minio_url, sha256, kind_str, _)) =
                        self.upload_dispute_media(dispute_id, media_ref).await
                    {
                        let _ = self.core.add_dispute_evidence_media(order_id, &minio_url, &sha256, &kind_str).await;
                    }
                }
                evidence_count.saturating_add(1)
            } else {
                let _ = self.core.add_dispute_evidence_text(order_id, session.user_id, &ev.body).await;
                evidence_count.saturating_add(1)
            };

            session.state = OrderFlowState::DisputeSellerResponding { order_id, evidence_count: new_count };
            session.touch();
            self.sessions.update(&peer, session).await;
            self.outbound.send_text(&peer, &message_templates::dispute_evidence_received(new_count)).await;
            return Ok(());
        }

        if let Some(order_id) = order_flow::try_dispute_order_id(&session.state, &ev.body) {
            // Abre disputa no backend e inicia coleta de motivo.
            tracing::info!(peer = %peer_hint, %order_id, "whatsapp: disputa iniciada");
            session.state = OrderFlowState::DisputeCollectingReason { order_id };
            session.touch();
            self.sessions.update(&peer, session.clone()).await;
            self.trigger_summary(&peer, session.user_id, Some(order_id), SummaryTrigger::DisputeOpened);
            self.outbound
                .send_text(&peer, message_templates::dispute_reason_menu())
                .await;
            return Ok(());
        }

        // ─── Disputa: escolha do motivo (menu 1-5) ───────────────────────────────
        if let OrderFlowState::DisputeCollectingReason { order_id } = session.state.clone() {
            let choice: Option<u8> = ev.body.trim().parse().ok();
            if let Some(n) = choice.filter(|&n| n >= 1 && n <= 5) {
                use apicash_disputes::DisputeReason;
                let reason = DisputeReason::from_menu_choice(n)
                    .unwrap_or(DisputeReason::Other);
                let reason_str = reason.to_str().to_string();

                // Chama API para abrir a disputa no backend.
                let _ = self.core.open_dispute(order_id, &reason_str).await;

                // Notifica o vendedor com prazo de resposta (72h).
                self.notify_seller_dispute_opened(order_id, &session).await;

                session.state = OrderFlowState::DisputeCollectingEvidence {
                    order_id,
                    reason: reason_str.clone(),
                    evidence_count: 0,
                };
                session.touch();
                self.sessions.update(&peer, session).await;
                self.outbound
                    .send_text(&peer, &message_templates::dispute_evidence_request(&reason_str))
                    .await;
                return Ok(());
            }
            // Input inválido
            self.outbound.send_text(&peer, "Responda com o número do motivo (1 a 5).").await;
            return Ok(());
        }

        // ─── Disputa: coleta de evidências (fotos/rastreio/"pronto") ─────────────
        if let OrderFlowState::DisputeCollectingEvidence { order_id, reason, evidence_count } = session.state.clone() {
            let trimmed = ev.body.trim().to_lowercase();
            let is_done = trimmed == "pronto" || trimmed == "ok" || trimmed == "enviei" || evidence_count >= 5;

            if is_done && ev.media.is_none() {
                // Submete evidências e aciona análise IA em background.
                let core = self.core.clone();
                tokio::spawn(async move {
                    let _ = core.trigger_dispute_analysis(order_id).await;
                });
                session.state = OrderFlowState::DisputeAwaitingDecision { order_id };
                session.touch();
                self.sessions.update(&peer, session).await;
                self.outbound
                    .send_text(&peer, message_templates::dispute_evidence_submitted())
                    .await;
                return Ok(());
            }

            let new_count = if let Some(ref media_ref) = ev.media {
                // Mídia (foto/vídeo): busca o dispute_id para upload MinIO.
                let disputes = self.core.get_dispute_for_order(order_id).await
                    .ok()
                    .flatten();
                if let Some(dispute_id) = disputes {
                    if let Some((minio_url, sha256, kind_str, _ext)) =
                        self.upload_dispute_media(dispute_id, media_ref).await
                    {
                        let _ = self.core.add_dispute_evidence_media(
                            order_id, &minio_url, &sha256, &kind_str,
                        ).await;
                        tracing::info!(%order_id, %dispute_id, kind = %kind_str, "dispute: media evidence added");
                    } else {
                        self.outbound.send_text(&peer, "⚠️ Não consegui processar a mídia. Envie novamente ou use 'pronto' para continuar.").await;
                    }
                }
                evidence_count.saturating_add(1)
            } else {
                // Texto/rastreio vira evidência.
                let _ = self.core.add_dispute_evidence_text(order_id, session.user_id, &ev.body).await;
                evidence_count.saturating_add(1)
            };

            session.state = OrderFlowState::DisputeCollectingEvidence {
                order_id,
                reason,
                evidence_count: new_count,
            };
            session.touch();
            self.sessions.update(&peer, session).await;

            if new_count >= 5 {
                // Auto-finaliza após 5 evidências.
                let core = self.core.clone();
                tokio::spawn(async move {
                    let _ = core.trigger_dispute_analysis(order_id).await;
                });
                let mut s = self.sessions.session_for(&peer).await;
                s.state = OrderFlowState::DisputeAwaitingDecision { order_id };
                s.touch();
                self.sessions.update(&peer, s).await;
                self.outbound.send_text(&peer, message_templates::dispute_evidence_submitted()).await;
            } else {
                self.outbound
                    .send_text(&peer, &message_templates::dispute_evidence_received(new_count))
                    .await;
            }
            return Ok(());
        }

        // ─── Disputa aguardando decisão ───────────────────────────────────────────
        if let OrderFlowState::DisputeAwaitingDecision { .. } = session.state {
            self.outbound.send_text(&peer, "Sua disputa está em análise. Você será notificado assim que houver uma decisão.").await;
            return Ok(());
        }

        if order_flow::is_accept_proposal(body) {
            if matches!(session.state, OrderFlowState::Idle) {
                if let Some(proposal) = self
                    .sessions
                    .find_pending_proposal_for_buyer(&peer)
                    .await
                {
                    self.sessions
                        .sync_buyer_proposal_peer(&peer, &proposal)
                        .await;
                    session.state = OrderFlowState::BuyerPendingSellerProposal {
                        seller_peer_key: proposal.seller_peer_key,
                        amount: proposal.amount,
                        description: proposal.description,
                    };
                }
            }
        }

        let prev_state = std::mem::replace(&mut session.state, OrderFlowState::Idle);
        match prev_state {
            OrderFlowState::BuyerPendingSellerProposal {
                seller_peer_key,
                amount,
                description,
            } => {
                if order_flow::peers_same_phone(&seller_peer_key, &peer) {
                    session.reset_flow();
                    self.sessions.update(&peer, session).await;
                    self.outbound
                        .send_text(&peer, message_templates::counterparty_same_as_sender())
                        .await;
                    return Ok(());
                }
                if order_flow::is_new_order(body) {
                    let spk = seller_peer_key.clone();
                    let amt = amount.clone();
                    let mut seller_reset = self.sessions.session_for(&spk).await;
                    if matches!(
                        seller_reset.state,
                        OrderFlowState::CreatingOrder {
                            step: CreatingOrderStep::WaitingBuyerAccept,
                            ..
                        }
                    ) {
                        seller_reset.reset_flow();
                        self.sessions.update(&spk, seller_reset).await;
                        self.outbound
                            .send_text(
                                &spk,
                                message_templates::seller_proposal_cancelled_by_buyer(&amt),
                            )
                            .await;
                    }
                    session.active_order_id = None;
                    self.start_holdfy_flow(&mut session, &peer, body, &ev).await?;
                    return Ok(());
                }
                if order_flow::is_reject_proposal(body) {
                    session.reset_flow();
                    self.sessions.update(&peer, session).await;
                    let mut seller_s = self.sessions.session_for(&seller_peer_key).await;
                    seller_s.reset_flow();
                    self.sessions.update(&seller_peer_key, seller_s).await;
                    self.outbound
                        .send_text(&peer, "**Recuso** — fluxo encerrado.")
                        .await;
                    self.outbound
                        .send_text(
                            &seller_peer_key,
                            message_templates::seller_buyer_refused(&amount),
                        )
                        .await;
                    return Ok(());
                }
                if order_flow::is_accept_proposal(body) {
                    return self
                        .begin_buyer_document_collection(
                            &mut session,
                            &peer,
                            seller_peer_key,
                            amount,
                            description,
                        )
                        .await;
                }

                session.state = OrderFlowState::BuyerPendingSellerProposal {
                    seller_peer_key,
                    amount,
                    description,
                };
                self.sessions.update(&peer, session).await;
                self.outbound
                    .send_text(&peer, "*ACEITO* = PIX | *RECUSO* / *não* = fim.")
                    .await;
                return Ok(());
            }
            OrderFlowState::AwaitingListingUrl { draft } => {
                self.handle_listing_url_response(&mut session, &peer, body, &ev, draft).await?;
            }
            OrderFlowState::Idle => {
                if let Some(tracking_code) = order_flow::extract_tracking_code(body) {
                    // Tenta resolver o order_id: primeiro da sessão, depois do payment_registry
                    // (o registry sobrevive a restarts; a sessão não).
                    let resolved_order_id = if let Some(oid) = session.active_order_id {
                        Some(oid)
                    } else {
                        self.payment_registry
                            .find_order_for_seller(&peer)
                            .await
                            .map(|(oid, _)| oid)
                    };

                    if let Some(order_id) = resolved_order_id {
                        // Restaura active_order_id na sessão para próximas mensagens.
                        if session.active_order_id.is_none() {
                            session.active_order_id = Some(order_id);
                            self.sessions.update(&peer, session.clone()).await;
                        }
                        self.register_seller_tracking(&peer, &tracking_code, order_id).await;
                    } else {
                        self.handle_tracking_request(&peer, &tracking_code, None).await;
                    }
                } else if order_flow::is_product_url(body) {
                    // URL enviada diretamente sem iniciar HoldFy → inicia o fluxo pedindo link
                    self.start_holdfy_flow(&mut session, &peer, body, &ev).await?;
                } else if order_flow::is_new_order(body) || body.eq_ignore_ascii_case("NOVO_PEDIDO") {
                    self.start_holdfy_flow(&mut session, &peer, body, &ev).await?;
                } else {
                    let welcome_msg = match session.contact_name.as_deref() {
                        Some(name) => message_templates::welcome_known(Self::first_name(name)),
                        None => message_templates::welcome().to_string(),
                    };
                    session.state = OrderFlowState::Idle;
                    self.sessions.update(&peer, session).await;
                    self.outbound.send_text(&peer, &welcome_msg).await;
                    self.outbound
                        .send_text(&peer, message_templates::menu_hint())
                        .await;
                    self.outbound
                        .send_welcome_interactive(&peer, &welcome_msg)
                        .await;
                }
            }
            OrderFlowState::CreatingOrder { step, draft } => {
                if matches!(step, CreatingOrderStep::AskAmount | CreatingOrderStep::AskCounterparty) {
                    self.advance_holdfy_collecting(&mut session, &peer, step, draft, body, &ev)
                        .await?;
                } else if matches!(step, CreatingOrderStep::AskSellerDocument) {
                    if let Some(doc) = order_flow::parse_document(body) {
                        self.accept_seller_document_and_propose(&mut session, &peer, draft, &doc)
                            .await?;
                    } else {
                        const MAX_DOC_ATTEMPTS: u32 = 5;
                        let attempt = session.bump_invalid();
                        session.last_activity_at = Utc::now();
                        if attempt >= MAX_DOC_ATTEMPTS {
                            session.reset_flow();
                            self.sessions.update(&peer, session).await;
                            self.outbound
                                .send_text(
                                    &peer,
                                    message_templates::invalid_document_too_many_attempts(),
                                )
                                .await;
                        } else {
                            session.state = OrderFlowState::CreatingOrder {
                                step: CreatingOrderStep::AskSellerDocument,
                                draft,
                            };
                            self.sessions.update(&peer, session).await;
                            self.outbound
                                .send_text(
                                    &peer,
                                    &message_templates::invalid_document_retry(
                                        attempt, MAX_DOC_ATTEMPTS,
                                    ),
                                )
                                .await;
                        }
                    }
                } else if matches!(step, CreatingOrderStep::AskSellerPix) {
                    if order_flow::is_pix_change(body) {
                        // Vendedor quer trocar a chave
                        session.seller_pix_key = None;
                        session.state = OrderFlowState::CreatingOrder {
                            step: CreatingOrderStep::AskSellerPix,
                            draft,
                        };
                        self.sessions.update(&peer, session).await;
                        self.outbound.send_text(&peer, message_templates::ask_seller_pix_key()).await;
                    } else if order_flow::is_pix_confirm(body) && session.seller_pix_key.is_some() {
                        // Confirma chave já guardada
                        let pix = session.seller_pix_key.clone().unwrap();
                        self.outbound.send_text(&peer, &message_templates::seller_pix_confirmed(&pix)).await;
                        self.try_send_holdfy_proposal(&mut session, &peer, draft).await.map(|_| ())?;
                    } else if let Some(pix) = order_flow::parse_pix_key(body) {
                        // Nova chave fornecida — guarda e avança
                        session.seller_pix_key = Some(pix.clone());
                        if let Some(pool) = &self.pg_pool {
                            let pool = pool.clone();
                            let peer_k = peer.clone();
                            let pix_k = pix.clone();
                            tokio::spawn(async move {
                                crate::wa_contact_store::save_pix_key(&pool, &peer_k, &pix_k).await;
                            });
                        }
                        self.outbound.send_text(&peer, &message_templates::seller_pix_confirmed(&pix)).await;
                        self.try_send_holdfy_proposal(&mut session, &peer, draft).await.map(|_| ())?;
                    } else {
                        // Formato inválido
                        session.state = OrderFlowState::CreatingOrder {
                            step: CreatingOrderStep::AskSellerPix,
                            draft,
                        };
                        self.sessions.update(&peer, session).await;
                        self.outbound.send_text(&peer, message_templates::seller_pix_invalid()).await;
                    }
                } else if matches!(step, CreatingOrderStep::WaitingBuyerAccept) {
                    if draft
                        .counterparty_peer_key
                        .as_ref()
                        .is_some_and(|bpk| order_flow::peers_same_phone(bpk, &peer))
                    {
                        session.reset_flow();
                        self.sessions.update(&peer, session).await;
                        self.outbound
                            .send_text(&peer, message_templates::counterparty_same_as_sender())
                            .await;
                        return Ok(());
                    }
                    if order_flow::is_accept_proposal(body) {
                        if let (Some(amt), Some(bpk)) =
                            (draft.amount.clone(), draft.counterparty_peer_key.clone())
                        {
                            let desc = draft
                                .description
                                .clone()
                                .unwrap_or_else(|| "HoldFy".into());
                            if order_flow::peers_same_phone(&bpk, &peer) {
                                let seller_doc = draft.seller_document.clone().unwrap_or_default();
                                if seller_doc.is_empty() {
                                    self.outbound
                                        .send_text(&peer, message_templates::ask_seller_document())
                                        .await;
                                    session.state = OrderFlowState::CreatingOrder {
                                        step: CreatingOrderStep::AskSellerDocument,
                                        draft,
                                    };
                                    self.sessions.update(&peer, session).await;
                                    return Ok(());
                                }
                                return self
                                    .begin_buyer_document_collection(
                                        &mut session,
                                        &peer,
                                        peer.clone(),
                                        amt,
                                        desc,
                                    )
                                    .await;
                            }
                        }
                        session.state = OrderFlowState::CreatingOrder {
                            step: CreatingOrderStep::WaitingBuyerAccept,
                            draft,
                        };
                        self.sessions.update(&peer, session).await;
                        self.outbound
                            .send_text(
                                &peer,
                                message_templates::seller_waiting_buyer_must_accept(),
                            )
                            .await;
                        return Ok(());
                    }
                    session.state = OrderFlowState::CreatingOrder {
                        step: CreatingOrderStep::WaitingBuyerAccept,
                        draft,
                    };
                    self.sessions.update(&peer, session).await;
                    self.outbound
                        .send_text(&peer, message_templates::seller_still_waiting_buyer())
                        .await;
                }
            }
            OrderFlowState::AwaitingBuyerDocument {
                seller_peer_key,
                seller_document,
                amount,
                description,
            } => {
                if let Some(doc) = order_flow::parse_document(body) {
                    self.finalize_order_after_buyer_accepted(
                        &seller_peer_key,
                        &peer,
                        amount,
                        description,
                        &seller_document,
                        &doc,
                        &peer_hint,
                    )
                    .await?;
                } else {
                    const MAX_DOC_ATTEMPTS: u32 = 5;
                    let attempt = session.bump_invalid();
                    session.last_activity_at = Utc::now();

                    if attempt >= MAX_DOC_ATTEMPTS {
                        let spk = seller_peer_key.clone();
                        session.reset_flow();
                        self.sessions.update(&peer, session).await;
                        // Libera também o vendedor se ainda estiver em WaitingBuyerAccept
                        let mut seller_s = self.sessions.session_for(&spk).await;
                        if matches!(
                            seller_s.state,
                            OrderFlowState::CreatingOrder {
                                step: CreatingOrderStep::WaitingBuyerAccept,
                                ..
                            }
                        ) {
                            seller_s.reset_flow();
                            self.sessions.update(&spk, seller_s).await;
                            self.outbound
                                .send_text(&spk, "Comprador não conseguiu enviar o CPF/CNPJ. Pedido cancelado.")
                                .await;
                        }
                        self.outbound
                            .send_text(&peer, message_templates::invalid_document_too_many_attempts())
                            .await;
                    } else {
                        session.state = OrderFlowState::AwaitingBuyerDocument {
                            seller_peer_key,
                            seller_document,
                            amount,
                            description,
                        };
                        self.sessions.update(&peer, session).await;
                        self.outbound
                            .send_text(
                                &peer,
                                &message_templates::invalid_document_retry(attempt, MAX_DOC_ATTEMPTS),
                            )
                            .await;
                    }
                }
            }
            OrderFlowState::AwaitingPayment {
                order_id,
                amount,
                description,
                pix_br_code,
            } => {
                if let Some(tracking_code) = order_flow::extract_tracking_code(body) {
                    session.state = OrderFlowState::AwaitingPayment {
                        order_id,
                        amount,
                        description,
                        pix_br_code,
                    };
                    self.sessions.update(&peer, session).await;
                    self.handle_tracking_request(&peer, &tracking_code, Some(order_id)).await;
                } else {
                    session.state = OrderFlowState::AwaitingPayment {
                        order_id,
                        amount,
                        description,
                        pix_br_code,
                    };
                    self.sessions.update(&peer, session).await;
                    self.outbound
                        .send_text(&peer, message_templates::awaiting_payment_hint())
                        .await;
                }
            }
            OrderFlowState::AwaitingConfirmation { order_id, amount, .. } => {
                if let Some(tracking_code) = order_flow::extract_tracking_code(body) {
                    session.state = OrderFlowState::AwaitingConfirmation {
                        order_id,
                        amount,
                        description: String::new(),
                    };
                    session.active_order_id = Some(order_id);
                    self.sessions.update(&peer, session).await;
                    self.handle_tracking_request(&peer, &tracking_code, Some(order_id)).await;
                } else if order_flow::is_confirm_receipt_final(body) {
                    // Confirmação explícita → release + off-ramp
                    self.handle_buyer_confirm_receipt(&peer, order_id, &amount, session).await;
                } else if order_flow::is_confirm_receipt_intent(body) {
                    // Primeiro toque ("recebi") — pede confirmação explícita com botões
                    session.state = OrderFlowState::AwaitingConfirmation {
                        order_id,
                        amount: amount.clone(),
                        description: String::new(),
                    };
                    session.active_order_id = Some(order_id);
                    self.sessions.update(&peer, session).await;
                    self.outbound
                        .send_interactive_confirm_receipt(
                            &peer,
                            &message_templates::ask_buyer_confirm_receipt(&order_id, &amount),
                        )
                        .await;
                } else {
                    // Qualquer outra mensagem: lembra o comprador do que precisa fazer, com botões.
                    session.state = OrderFlowState::AwaitingConfirmation {
                        order_id,
                        amount: amount.clone(),
                        description: String::new(),
                    };
                    session.active_order_id = Some(order_id);
                    self.sessions.update(&peer, session).await;
                    self.outbound
                        .send_interactive_confirm_receipt(
                            &peer,
                            &message_templates::ask_buyer_confirm_receipt(&order_id, &amount),
                        )
                        .await;
                }
            }
            OrderFlowState::DisputeCollectingReason { order_id } => {
                session.state = OrderFlowState::DisputeCollectingReason { order_id };
                self.sessions.update(&peer, session).await;
                self.outbound
                    .send_text(&peer, message_templates::dispute_reason_menu())
                    .await;
            }
            OrderFlowState::DisputeCollectingEvidence { order_id, reason, evidence_count } => {
                session.state = OrderFlowState::DisputeCollectingEvidence { order_id, reason, evidence_count };
                self.sessions.update(&peer, session).await;
                self.outbound.send_text(&peer, "Envie as fotos/evidências ou digite *pronto* para encerrar.").await;
            }
            OrderFlowState::DisputeAwaitingDecision { order_id } => {
                session.state = OrderFlowState::DisputeAwaitingDecision { order_id };
                self.sessions.update(&peer, session).await;
                self.outbound.send_text(&peer, "Sua disputa está em análise. Você será notificado quando houver uma decisão.").await;
            }
            OrderFlowState::DisputeSellerResponding { order_id, evidence_count } => {
                session.state = OrderFlowState::DisputeSellerResponding { order_id, evidence_count };
                self.sessions.update(&peer, session).await;
                self.outbound.send_text(&peer, "Envie suas contra-evidências ou digite *pronto* para encerrar.").await;
            }
        }

        Ok(())
    }
}

/// Baixa bytes de uma URL (imagem MinIO ou externa). Soft-fail: retorna Err em qualquer falha.
async fn download_url_bytes(url: &str) -> Result<Vec<u8>, String> {
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.bytes().await.map(|b| b.to_vec()).map_err(|e| e.to_string())
}

/// Baixa mídia da Cloud API (WhatsApp Business): resolve media_id → URL → bytes.
/// Requer `WHATSAPP_ACCESS_TOKEN` no ambiente.
async fn download_cloud_media(media_id: &str) -> Result<Vec<u8>, String> {
    let token = std::env::var("WHATSAPP_ACCESS_TOKEN")
        .map_err(|_| "WHATSAPP_ACCESS_TOKEN not set".to_string())?;

    let client = reqwest::Client::new();

    // Passo 1: resolve a URL de download.
    let meta_url = format!("https://graph.facebook.com/v20.0/{media_id}");
    let info: serde_json::Value = client
        .get(&meta_url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| format!("graph API request: {e}"))?
        .json()
        .await
        .map_err(|e| format!("graph API json: {e}"))?;

    let download_url = info["url"]
        .as_str()
        .ok_or_else(|| format!("graph API: no url in response: {info}"))?
        .to_string();

    // Passo 2: baixa o arquivo.
    let bytes = client
        .get(&download_url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| format!("media download: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("media bytes: {e}"))?
        .to_vec();

    Ok(bytes)
}
