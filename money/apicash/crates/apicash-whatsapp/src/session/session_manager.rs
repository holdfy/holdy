//! Sessões por usuário (telefone / JID) e estado do fluxo de pedido.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::handlers::order_flow;

/// Estado da conversa de pedido protegido.
#[derive(Debug, Clone)]
pub enum OrderFlowState {
    Idle,
    /// Guided, multi-step order creation.
    CreatingOrder {
        step: CreatingOrderStep,
        draft: OrderDraft,
    },
    /// Comprador (B) deve aceitar/recusar o valor antes de gerar PIX.
    BuyerPendingSellerProposal {
        /// `peer_id` dígitos do vendedor (A), para validar quando B responde.
        seller_peer_key: String,
        amount: String,
        description: String,
    },
    /// Comprador (B) aceitou; aguardamos CPF/CNPJ do comprador (vendedor já informou o dele).
    AwaitingBuyerDocument {
        seller_peer_key: String,
        seller_document: String,
        amount: String,
        description: String,
    },
    /// Pedido criado e PIX gerado; aguarda o comprador efetuar o pagamento.
    AwaitingPayment {
        order_id: Uuid,
        amount: String,
        description: String,
        pix_br_code: String,
    },
    /// Usuário declarou que pagou; aguarda confirmação explícita de recebimento do produto.
    AwaitingConfirmation {
        order_id: Uuid,
        amount: String,
        description: String,
    },
    /// Vendedor (A) iniciou HoldFy; aguarda link do anúncio antes de continuar.
    AwaitingListingUrl {
        /// Rascunho parcial (pode já ter amount/phone se vieram na frase inicial).
        draft: OrderDraft,
    },
    /// Disputa aberta: aguardando o comprador escolher o motivo (menu 1-5).
    DisputeCollectingReason {
        order_id: Uuid,
    },
    /// Comprador escolheu o motivo; aguardando envio de fotos/rastreio/texto.
    DisputeCollectingEvidence {
        order_id:  Uuid,
        reason:    String,
        /// Número de evidências já enviadas nesta sessão.
        evidence_count: u8,
    },
    /// Evidências submetidas; disputa aguarda análise IA / resposta do vendedor / admin.
    DisputeAwaitingDecision {
        order_id: Uuid,
    },
}

/// Step within the guided order creation flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatingOrderStep {
    /// Comprador (parte B): número digitado ou cartão de contacto.
    AskCounterparty,
    AskAmount,
    /// Vendedor (A): CPF/CNPJ antes de enviar proposta ao comprador.
    AskSellerDocument,
    /// Vendedor (A): chave PIX para receber o pagamento após confirmação do comprador.
    AskSellerPix,
    /// Proposta enviada a B; aguardamos *ACEITO* antes do `POST /orders`.
    WaitingBuyerAccept,
}

/// Draft collected from the user before creating an order in the API.
#[derive(Debug, Clone, Default)]
pub struct OrderDraft {
    /// Peer key normalizada (só dígitos, sem `+`) do comprador — parte B.
    pub counterparty_peer_key: Option<String>,
    pub amount: Option<String>,
    pub description: Option<String>,
    /// CPF/CNPJ do vendedor (quem iniciou o HoldFy).
    pub seller_document: Option<String>,
    /// UUID do listing importado (salvo no banco pelo core).
    pub listing_id: Option<Uuid>,
    /// Fotos do anúncio (URLs MinIO). Primeira = imagem principal.
    pub listing_photos: Vec<String>,
    /// URL original do anúncio (Instagram, ML, etc.).
    pub listing_source_url: Option<String>,
    /// Preço sugerido extraído do anúncio (pode diferir do valor cobrado pelo vendedor).
    pub listing_price_suggested: Option<String>,
}

/// `Uuid` estável por peer WhatsApp (igual ao da sessão desse número).
#[must_use]
pub fn user_id_for_peer_key(peer_key: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_DNS,
        format!("apicash:whatsapp:user:{peer_key}").as_bytes(),
    )
}

#[derive(Debug, Clone)]
pub struct UserSession {
    pub peer_id: String,
    /// Stable user identity for this WhatsApp peer.
    ///
    /// Security rule: this `user_id` must be the same value used as `buyer_id` when creating
    /// escrow orders, and must be the identity sent to `POST /custody/release` on confirmation.
    pub user_id: Uuid,
    /// Last/active order created via this session (defense-in-depth for confirmation).
    pub active_order_id: Option<Uuid>,
    pub state: OrderFlowState,
    /// Última atualização (reseta TTL lógico).
    pub last_activity_at: DateTime<Utc>,
    /// Tentativas de input inválido no estado atual (ex.: valor mal formatado).
    pub invalid_input_streak: u32,
    /// Nome obtido da Receita Federal após consulta de CPF/CNPJ.
    pub contact_name: Option<String>,
    /// CPF ou CNPJ fornecido durante o fluxo (armazenado em wa_contacts).
    pub document: Option<String>,
    /// Chave PIX do vendedor (coletada durante criação do pedido, persistida em wa_contacts).
    pub seller_pix_key: Option<String>,
    /// Indica se já tentámos carregar o contato do banco nesta sessão.
    pub contact_loaded: bool,
}

impl UserSession {
    pub fn new(peer_id: impl Into<String>) -> Self {
        let peer_id = peer_id.into();
        let user_id = user_id_for_peer_key(&peer_id);
        Self {
            peer_id,
            user_id,
            active_order_id: None,
            state: OrderFlowState::Idle,
            last_activity_at: Utc::now(),
            invalid_input_streak: 0,
            contact_name: None,
            document: None,
            seller_pix_key: None,
            contact_loaded: false,
        }
    }

    /// Reseta o fluxo para o estado inicial.
    pub fn reset_flow(&mut self) {
        self.state = OrderFlowState::Idle;
        self.active_order_id = None;
        self.touch();
    }

    pub fn touch(&mut self) {
        self.last_activity_at = Utc::now();
        self.invalid_input_streak = 0;
    }

    pub fn reset_invalid_streak(&mut self) {
        self.invalid_input_streak = 0;
    }

    pub fn bump_invalid(&mut self) -> u32 {
        self.invalid_input_streak = self.invalid_input_streak.saturating_add(1);
        self.invalid_input_streak
    }
}

/// Proposta pendente para o comprador aceitar (B).
#[derive(Debug, Clone)]
pub struct PendingBuyerProposal {
    pub seller_peer_key: String,
    pub amount: String,
    pub description: String,
}

/// Armazenamento em memória (substituir por Redis/Postgres em produção).
#[derive(Clone, Default)]
pub struct SessionManager {
    inner: Arc<RwLock<HashMap<String, UserSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// TTL máximo sem atividade antes de sugerir reset (não bloqueia — só mensagem).
    #[must_use]
    pub fn idle_timeout_hint() -> Duration {
        Duration::from_secs(60 * 45)
    }

    pub async fn session_for(&self, peer_id: &str) -> UserSession {
        let mut map = self.inner.write().await;
        map.entry(peer_id.to_string())
            .or_insert_with(|| UserSession::new(peer_id))
            .clone()
    }

    pub async fn update(&self, peer_id: &str, session: UserSession) {
        let mut map = self.inner.write().await;
        map.insert(peer_id.to_string(), session);
    }

    pub async fn reset(&self, peer_id: &str) {
        let mut map = self.inner.write().await;
        map.remove(peer_id);
    }

    /// Localiza proposta pendente quando a chave do peer do comprador difere (PN vs LID / dígito).
    pub async fn find_pending_proposal_for_buyer(
        &self,
        buyer_peer: &str,
    ) -> Option<PendingBuyerProposal> {
        let map = self.inner.read().await;
        for sess in map.values() {
            if let OrderFlowState::BuyerPendingSellerProposal {
                seller_peer_key,
                amount,
                description,
            } = &sess.state
            {
                if order_flow::peers_same_phone(&sess.peer_id, buyer_peer) {
                    return Some(PendingBuyerProposal {
                        seller_peer_key: seller_peer_key.clone(),
                        amount: amount.clone(),
                        description: description.clone(),
                    });
                }
            }
        }
        for sess in map.values() {
            if let OrderFlowState::CreatingOrder {
                step: CreatingOrderStep::WaitingBuyerAccept,
                draft,
            } = &sess.state
            {
                if draft.counterparty_peer_key.as_ref().is_some_and(|bpk| {
                    order_flow::peers_same_phone(bpk, buyer_peer)
                }) {
                    return Some(PendingBuyerProposal {
                        seller_peer_key: sess.peer_id.clone(),
                        amount: draft.amount.clone().unwrap_or_else(|| "?".into()),
                        description: draft
                            .description
                            .clone()
                            .unwrap_or_else(|| "HoldFy".into()),
                    });
                }
            }
        }
        None
    }

    /// Garante estado `BuyerPendingSellerProposal` na chave de peer que acabou de falar.
    pub async fn sync_buyer_proposal_peer(
        &self,
        buyer_peer: &str,
        proposal: &PendingBuyerProposal,
    ) {
        let mut sess = self.session_for(buyer_peer).await;
        sess.state = OrderFlowState::BuyerPendingSellerProposal {
            seller_peer_key: proposal.seller_peer_key.clone(),
            amount: proposal.amount.clone(),
            description: proposal.description.clone(),
        };
        sess.touch();
        self.update(buyer_peer, sess).await;
    }

    /// Resolve vendedor/comprador/valor a partir de sessões com `active_order_id` (fallback após restart).
    pub async fn find_parties_by_order_id(&self, order_id: Uuid) -> Option<crate::payment_notify::OrderPaymentParties> {
        let map = self.inner.read().await;
        let mut seller_peer: Option<String> = None;
        let mut buyer_peer: Option<String> = None;
        let mut amount: Option<String> = None;

        for sess in map.values() {
            if sess.active_order_id != Some(order_id) {
                continue;
            }
            match &sess.state {
                OrderFlowState::AwaitingPayment {
                    amount: a, ..
                }
                | OrderFlowState::AwaitingConfirmation {
                    amount: a, ..
                } => {
                    buyer_peer = Some(sess.peer_id.clone());
                    amount = Some(a.clone());
                }
                OrderFlowState::Idle => {
                    seller_peer = Some(sess.peer_id.clone());
                }
                OrderFlowState::BuyerPendingSellerProposal {
                    seller_peer_key,
                    amount: a,
                    ..
                } => {
                    buyer_peer = Some(sess.peer_id.clone());
                    seller_peer = Some(seller_peer_key.clone());
                    amount = Some(a.clone());
                }
                _ => {}
            }
        }

        let seller = seller_peer?;
        let buyer = buyer_peer?;
        if seller == buyer {
            return None;
        }
        Some(crate::payment_notify::OrderPaymentParties {
            seller_peer: seller,
            buyer_peer: buyer,
            amount: amount.unwrap_or_else(|| "?".into()),
        })
    }
}
