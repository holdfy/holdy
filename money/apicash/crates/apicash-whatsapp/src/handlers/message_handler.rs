//! Orquestra mensagens de texto e o fluxo de pedido.

use std::sync::Arc;

use apicash_auth::{AuthConfig, AuthService, Role};
use apicash_importer::ImporterService;
use apicash_logistics::LogisticsService;
use chrono::Utc;
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
    importer: Arc<ImporterService>,
    conv_store: Arc<ConversationStore>,
    logistics: Arc<LogisticsService>,
}

impl MessageHandler {
    pub fn new(
        core: CoreApiClient,
        outbound: Arc<Outbound>,
        sessions: Arc<SessionManager>,
        payment_registry: Arc<PaymentNotifyRegistry>,
        importer: Arc<ImporterService>,
        conv_store: Arc<ConversationStore>,
        logistics: Arc<LogisticsService>,
    ) -> Self {
        Self {
            core,
            outbound,
            sessions,
            payment_registry,
            jwt: AuthService::new(AuthConfig::from_env()),
            importer,
            conv_store,
            logistics,
        }
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

    /// Pagamento confirmado no Gatebox: avisa vendedor e comprador (sem settle nem liberação de custódia).
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
        self.payment_registry.mark_notified(order_id).await;

        // Gerar resumo quando pagamento confirmado
        let buyer_uid = crate::session::user_id_for_peer_key(&parties.buyer_peer);
        let seller_uid = crate::session::user_id_for_peer_key(&parties.seller_peer);
        self.trigger_summary(&parties.buyer_peer, buyer_uid, Some(order_id), SummaryTrigger::PaymentConfirmed);
        self.trigger_summary(&parties.seller_peer, seller_uid, Some(order_id), SummaryTrigger::PaymentConfirmed);

        let mut buyer_sess = self.sessions.session_for(&parties.buyer_peer).await;
        buyer_sess.reset_flow();
        buyer_sess.active_order_id = Some(order_id);
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

        session.state = OrderFlowState::CreatingOrder {
            step: CreatingOrderStep::WaitingBuyerAccept,
            draft: OrderDraft {
                counterparty_peer_key: Some(buyer_peer_key.clone()),
                amount: Some(amt.clone()),
                description: Some(description.clone()),
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
        let buyer_ok = self
            .outbound
            .send_text(
                &buyer_peer_key,
                message_templates::buyer_proposal_before_accept(&masked_seller, &amt, &description),
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

    /// Vendedor enviou um link de produto — importar e pré-preencher rascunho.
    async fn handle_url_import(
        &self,
        session: &mut UserSession,
        seller_peer: &str,
        url: &str,
    ) -> Result<(), CoreApiError> {
        self.outbound
            .send_text(seller_peer, message_templates::importing_product())
            .await;

        match self.importer.import(url).await {
            Ok(draft_product) => {
                let title = draft_product.title.clone();
                let price_str = draft_product.price_suggested.map(|p| p.round_dp(2).normalize().to_string());

                let mut draft = OrderDraft {
                    description: Some(title.clone()),
                    ..Default::default()
                };

                if let Some(ref price) = price_str {
                    draft.amount = Some(price.clone());
                    session.state = OrderFlowState::CreatingOrder {
                        step: CreatingOrderStep::AskCounterparty,
                        draft,
                    };
                    session.touch();
                    self.sessions.update(seller_peer, session.clone()).await;
                    self.outbound
                        .send_text(seller_peer, &message_templates::product_imported_with_price(&title, price))
                        .await;
                } else {
                    session.state = OrderFlowState::CreatingOrder {
                        step: CreatingOrderStep::AskAmount,
                        draft,
                    };
                    session.touch();
                    self.sessions.update(seller_peer, session.clone()).await;
                    self.outbound
                        .send_text(seller_peer, &message_templates::product_imported_no_price(&title))
                        .await;
                }
            }
            Err(e) => {
                tracing::warn!(peer = %mask_whatsapp_peer(seller_peer), url_len = url.len(), error = %e, "importer: URL import failed");
                session.state = OrderFlowState::Idle;
                self.sessions.update(seller_peer, session.clone()).await;
                self.outbound
                    .send_text(seller_peer, message_templates::product_import_failed())
                    .await;
            }
        }
        Ok(())
    }

    async fn start_holdfy_flow(
        &self,
        session: &mut UserSession,
        seller_peer: &str,
        body: &str,
        ev: &WhatsAppEvent,
    ) -> Result<(), CoreApiError> {
        let mut draft = OrderDraft::default();
        if let Some(parsed) = parse_holdfy_message(body, Some(ev)) {
            draft.amount = parsed.amount;
            draft.counterparty_peer_key = parsed.phone;
        } else {
            Self::merge_holdfy_draft(&mut draft, body, ev);
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

        session.active_order_id = None;
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

    /// Após comprador responder *ACEITO* e fornecer documento: criar pedido, enviar PIX a B e avisos a A/B.
    async fn finalize_order_after_buyer_accepted(
        &self,
        seller_peer_key: &str,
        buyer_peer_key: &str,
        amount: String,
        description: String,
        document: &str,
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
            .calculate_risk_score_internal_only(buyer_id, document, social_empty)
            .await
        {
            tracing::warn!(
                peer = %peer_hint_buyer,
                buyer_id = %buyer_id,
                error = %e,
                "core_api: calculate_risk_score_internal_only failed (continuing). Configure APICASH_API_KEY for antifraude."
            );
        }

        // Consulta nome do comprador via NFS-e (não bloqueia se falhar).
        let buyer_name = crate::nfse_client::lookup_name(document).await;
        if let Some(ref name) = buyer_name {
            let msg = format!("*Comprador identificado:* {name}");
            self.outbound.send_text(buyer_peer_key, &msg).await;
            self.outbound.send_text(seller_peer_key, &msg).await;
        }

        let order = match self
            .core
            .create_order(
                buyer_id,
                seller_id,
                &amount,
                document,
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
            self.outbound
                .send_text(&peer, message_templates::welcome())
                .await;
            self.outbound
                .send_text(&peer, message_templates::welcome_help())
                .await;
            return Ok(());
        }

        if let Some(next) = order_flow::try_dispute(&session.state, &ev.body) {
            let dispute_order_id = match &next { OrderFlowState::DisputeHint { order_id } => Some(*order_id), _ => None };
            session.state = next;
            tracing::info!(
                peer = %peer_hint,
                user_id = %session.user_id,
                action = "DisputeOpened",
                success = true,
                "whatsapp: disputa solicitada"
            );
            self.sessions.update(&peer, session.clone()).await;
            self.trigger_summary(&peer, session.user_id, dispute_order_id, SummaryTrigger::DisputeOpened);
            self.outbound
                .send_text(&peer, message_templates::dispute_message())
                .await;
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
                    session.state = OrderFlowState::AwaitingBuyerDocument {
                        seller_peer_key,
                        amount,
                        description,
                    };
                    session.touch();
                    self.sessions.update(&peer, session).await;
                    self.outbound
                        .send_text(&peer, message_templates::ask_buyer_document())
                        .await;
                    return Ok(());
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
            OrderFlowState::Idle => {
                if let Some(tracking_code) = order_flow::extract_tracking_code(body) {
                    self.handle_tracking_request(&peer, &tracking_code, session.active_order_id).await;
                } else if order_flow::is_product_url(body) {
                    self.handle_url_import(&mut session, &peer, body).await?;
                } else if order_flow::is_new_order(body) || body.eq_ignore_ascii_case("NOVO_PEDIDO") {
                    self.start_holdfy_flow(&mut session, &peer, body, &ev).await?;
                } else {
                    session.state = OrderFlowState::Idle;
                    self.sessions.update(&peer, session).await;
                    self.outbound
                        .send_text(&peer, message_templates::welcome())
                        .await;
                    self.outbound
                        .send_text(&peer, message_templates::menu_hint())
                        .await;
                    self.outbound
                        .send_welcome_interactive(&peer, message_templates::welcome())
                        .await;
                }
            }
            OrderFlowState::CreatingOrder { step, draft } => {
                if matches!(step, CreatingOrderStep::AskAmount | CreatingOrderStep::AskCounterparty) {
                    self.advance_holdfy_collecting(&mut session, &peer, step, draft, body, &ev)
                        .await?;
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
                                session.state = OrderFlowState::AwaitingBuyerDocument {
                                    seller_peer_key: peer.clone(),
                                    amount: amt,
                                    description: desc,
                                };
                                session.touch();
                                self.sessions.update(&peer, session).await;
                                self.outbound
                                    .send_text(&peer, message_templates::ask_buyer_document())
                                    .await;
                                return Ok(());
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
                amount,
                description,
            } => {
                if let Some(doc) = order_flow::parse_document(body) {
                    self.finalize_order_after_buyer_accepted(
                        &seller_peer_key,
                        &peer,
                        amount,
                        description,
                        &doc,
                        &peer_hint,
                    )
                    .await?;
                } else {
                    session.state = OrderFlowState::AwaitingBuyerDocument {
                        seller_peer_key,
                        amount,
                        description,
                    };
                    session.touch();
                    self.sessions.update(&peer, session).await;
                    self.outbound
                        .send_text(&peer, message_templates::invalid_document())
                        .await;
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
            OrderFlowState::AwaitingConfirmation { order_id, .. } => {
                if let Some(tracking_code) = order_flow::extract_tracking_code(body) {
                    session.reset_flow();
                    session.active_order_id = Some(order_id);
                    self.sessions.update(&peer, session).await;
                    self.handle_tracking_request(&peer, &tracking_code, Some(order_id)).await;
                } else {
                    session.reset_flow();
                    session.active_order_id = Some(order_id);
                    self.sessions.update(&peer, session).await;
                    self.outbound
                        .send_text(&peer, message_templates::awaiting_payment_hint())
                        .await;
                }
            }
            OrderFlowState::DisputeHint { order_id } => {
                session.state = OrderFlowState::DisputeHint { order_id };
                self.sessions.update(&peer, session).await;
                self.outbound
                    .send_text(&peer, message_templates::dispute_message())
                    .await;
            }
        }

        Ok(())
    }
}
