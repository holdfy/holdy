// MessageHandler for payment queue: fetches tx, calls gateway, updates status.
use std::sync::Arc;

use async_trait::async_trait;

use crate::core::gateway_failover::{GatewayRecorder, GatewaySelector};
use crate::core::gateways::interfaces::sulcred::SulcredSendPixPayment;
use crate::core::gateways::services::GatewayHttpService;
use crate::internal::anchor::payload_hash_pix_tx;
use crate::internal::anchor::{EntityType, PublishRequest};
use crate::transaction::TransactionRepository;

use super::interfaces::MessageHandler;
use super::PaymentMessage;

/// Stub handler: logs and acks (used when no gateway configured).
pub struct PaymentMessageHandlerStub;

impl PaymentMessageHandlerStub {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PaymentMessageHandlerStub {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageHandler for PaymentMessageHandlerStub {
    async fn handle(&self, msg: PaymentMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!(
            payment_id = msg.payment_id,
            amount = msg.amount,
            "payment message processed (stub ack)"
        );
        Ok(())
    }
}

/// Gateway config for multi-gateway handler
pub struct GatewayConfig {
    pub name: String,
    pub service: Arc<dyn GatewayHttpService>,
    pub client_id: String,
    pub client_secret: String,
}

/// Real handler: fetches transaction, calls gateway, updates status.
/// Supports GatewaySelector for failover when multiple gateways configured.
pub struct PaymentMessageHandler {
    transaction_repo: Arc<dyn TransactionRepository>,
    gateways: Vec<GatewayConfig>,
    default_gateway_name: String,
    gateway_selector: Option<Arc<dyn GatewaySelector>>,
    anchor_publisher: Option<Arc<dyn crate::internal::anchor::AnchorPublisher>>,
    gateway_recorder: Option<Arc<dyn GatewayRecorder>>,
}

impl PaymentMessageHandler {
    pub fn new(
        transaction_repo: Arc<dyn TransactionRepository>,
        gateway: Arc<dyn GatewayHttpService>,
        client_id: String,
        client_secret: String,
        gateway_name: String,
    ) -> Self {
        Self {
            transaction_repo,
            gateways: vec![GatewayConfig {
                name: gateway_name.clone(),
                service: gateway,
                client_id,
                client_secret,
            }],
            default_gateway_name: gateway_name,
            gateway_selector: None,
            anchor_publisher: None,
            gateway_recorder: None,
        }
    }

    /// Add a second gateway for failover (use with with_gateway_selector)
    pub fn with_fallback_gateway(
        mut self,
        gateway: Arc<dyn GatewayHttpService>,
        client_id: String,
        client_secret: String,
        gateway_name: String,
    ) -> Self {
        self.gateways.push(GatewayConfig {
            name: gateway_name,
            service: gateway,
            client_id,
            client_secret,
        });
        self
    }

    pub fn with_gateway_selector(mut self, selector: Arc<dyn GatewaySelector>) -> Self {
        self.gateway_selector = Some(selector);
        self
    }

    pub fn with_anchor_publisher(mut self, publisher: Arc<dyn crate::internal::anchor::AnchorPublisher>) -> Self {
        self.anchor_publisher = Some(publisher);
        self
    }

    pub fn with_gateway_recorder(mut self, gr: Arc<dyn GatewayRecorder>) -> Self {
        self.gateway_recorder = Some(gr);
        self
    }

    fn resolve_gateway(&self) -> &GatewayConfig {
        self.gateways.first().expect("at least one gateway")
    }

    async fn resolve_gateway_async(&self) -> &GatewayConfig {
        if let Some(ref sel) = self.gateway_selector {
            if let Some(gw_name) = sel.select_gateway(&self.default_gateway_name).await {
                if let Some(cfg) = self.gateways.iter().find(|g| g.name == gw_name) {
                    return cfg;
                }
            }
        }
        self.resolve_gateway()
    }
}

#[async_trait]
impl MessageHandler for PaymentMessageHandler {
    async fn handle(&self, msg: PaymentMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Processing payment {} (amount: {:.2})", msg.payment_id, msg.amount);

        let gw = self.resolve_gateway_async().await;
        let gateway_name = &gw.name;

        let tx = self
            .transaction_repo
            .get_by_id(msg.payment_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("transaction {} not found", msg.payment_id))?;

        if tx.type_transaction_id != 1 {
            tracing::warn!(
                "Payment {} type_transaction_id={} (not PIX OUT), skipping",
                msg.payment_id,
                tx.type_transaction_id
            );
            return Ok(());
        }

        // Update to QUEUED
        self.transaction_repo
            .update_pix_status(msg.payment_id, 2, "", gateway_name)
            .await
            .map_err(|e| anyhow::anyhow!("update status: {}", e))?;

        // msg.amount = net_amount (what gateway sends to recipient); tx.amount = total debit
        let amount_f = msg.amount;

        let token_resp = gw
            .service
            .get_token_out(&gw.client_id, &gw.client_secret)
            .await?;
        let token = token_resp.access_token;

        let gw_request = crate::core::gateways::interfaces::sulcred::SulcredSendPixKeyRequest {
            expiration: 3600,
            pix_key: tx.key.clone(),
            priority: None,
            creditor_document: Some(tx.document_number.clone()),
            description: tx.remittance_information.clone(),
            payment: SulcredSendPixPayment {
                currency: "BRL".to_string(),
                amount: amount_f,
            },
            internal_transaction_id: Some(tx.external_id.clone()),
        };

        match gw
            .service
            .send_pix_key(&token, &gw_request, msg.failure_configs.as_ref().and_then(|m| m.get(gateway_name)))
            .await
        {
            Ok(gw_resp) => {
                tracing::info!("Payment {} completed via {} (endToEndId: {})", msg.payment_id, gateway_name, gw_resp.end_to_end_id);
                if !gw_resp.end_to_end_id.is_empty() {
                    self.transaction_repo
                        .update_pix_status_with_endtoend(msg.payment_id, 4, "", gateway_name, &gw_resp.end_to_end_id)
                        .await?;
                } else {
                    self.transaction_repo
                        .update_pix_status(msg.payment_id, 4, "", gateway_name)
                        .await?;
                }
                if let Some(ref gr) = self.gateway_recorder {
                    gr.record_success(gateway_name).await;
                }
                // Ancoragem blockchain (fire-and-forget)
                if let Some(ref publisher) = self.anchor_publisher {
                    let account_id = tx.account_id;
                    let endtoend = gw_resp.end_to_end_id.clone();
                    let publisher = Arc::clone(publisher);
                    let occurred_at = chrono::Utc::now();
                    let entity_id = msg.payment_id.to_string();
                    let payload_hash = payload_hash_pix_tx(
                        &entity_id,
                        amount_f,
                        &endtoend,
                        &occurred_at.to_rfc3339(),
                        account_id,
                        "COMPLETED",
                    );
                    let req = PublishRequest {
                        idempotency_key: format!("pix_tx_{}", msg.payment_id),
                        entity_type: EntityType::PixTx,
                        entity_id: entity_id.clone(),
                        payload_hash,
                        occurred_at,
                        correlation_id: String::new(),
                        account_id,
                        customer_id: None,
                        company_id: None,
                        actor_document: String::new(),
                        actor_name: String::new(),
                        actor_type: String::new(),
                        client_ip: String::new(),
                        user_agent: String::new(),
                        metadata: None,
                    };
                    let entity_id_log = entity_id.clone();
                    tokio::task::spawn_blocking(move || {
                        if let Err(e) = publisher.publish_anchor_request(&req) {
                            tracing::error!("Anchor publish failed for transaction {}: {}", entity_id_log, e);
                        }
                    });
                }
            }
            Err(e) => {
                tracing::error!("Payment {} failed: {}", msg.payment_id, e);
                if let Some(ref gr) = self.gateway_recorder {
                    gr.record_error(
                        gateway_name,
                        tx.account_id,
                        msg.payment_id,
                        "payment_processing_error",
                        &e.to_string(),
                    )
                    .await;
                }
                self.transaction_repo
                    .update_pix_status(msg.payment_id, 7, &e.to_string(), gateway_name)
                    .await?;
                return Err(e.into());
            }
        }

        Ok(())
    }
}
