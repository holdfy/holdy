// Webhook PIX IN/OUT and SendReversal - from gateboxgo ReceivePixIn, ReceivePixOut, SendReversal
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::account_rules::AccountRulesRepository;
use crate::accounts::AccountsRepository;
use crate::core::gateway_failover::GatewayRecorder;
use crate::core::pix_principal::ProviderSelector;
use crate::core::pix_principal::WebhookBatchProcessor;
use crate::fees::FeesRepository;
use crate::invoice::InvoiceRepository;
use crate::key_pix::KeyPixRepository;
use crate::model::SecMed;
use crate::model::Transaction;
use crate::sec_med::SecMedRepository;
use crate::transaction::TransactionRepository;
use crate::with_list_accounts::WithListAccountsRepository;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivePixInRequest {
    pub end_to_end_id: String,
    pub amount: f64,
    pub pix_key: String,
    #[serde(rename = "isQRCodePayment", default)]
    pub is_qr_code_payment: bool,
    #[serde(default)]
    pub payer_name: String,
    #[serde(default)]
    pub payer_document: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub idempotency_key: String,
    #[serde(default)]
    pub gateway: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivePixInResponse {
    pub status_code: i32,
    pub transaction_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivePixOutRequest {
    #[serde(default)]
    pub transaction_id: String,
    pub end_to_end_id: String,
    pub status: String,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub gateway_name: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub error_code: String,
    #[serde(default)]
    pub internal_transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivePixOutResponse {
    pub status_code: i32,
    pub transaction_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendReversalRequest {
    pub end2end: String,
    pub amount: f64,
    #[serde(default)]
    pub external_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendReversalResponse {
    pub status_code: i32,
    pub transaction_id: String,
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

#[async_trait]
pub trait PixWebhookService: Send + Sync {
    async fn receive_pix_in(&self, req: ReceivePixInRequest) -> Result<ReceivePixInResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn receive_pix_out(&self, req: ReceivePixOutRequest) -> Result<ReceivePixOutResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn send_reversal(&self, user_id: i64, req: SendReversalRequest) -> Result<SendReversalResponse, Box<dyn std::error::Error + Send + Sync>>;
}

fn normalize_gateway(name: &str) -> String {
    let s = name.trim().to_lowercase();
    if s.is_empty() {
        return String::new();
    }
    match s.as_str() {
        "seventrust" | "seven_trust" | "7trust" => "seventrust".to_string(),
        "sulcred" => "sulcred".to_string(),
        _ => s,
    }
}

fn capitalize_initial(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) => first.to_uppercase().chain(c).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_numbers(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn generate_end_to_end() -> String {
    let now = Utc::now();
    format!("E{}{:06}", now.format("%Y%m%d%H%M%S"), now.timestamp_subsec_micros() % 1_000_000)
}

/// Fee calculation for PIX IN (Go: fixed, percent, markup, net_amount).
#[allow(dead_code)]
fn calculate_fee_pix_in(amount: f64, fixed_cash_in: f64, percent_cashin: f64) -> (f64, f64) {
    let markup = 1.0 - percent_cashin / 100.0;
    let net = amount * markup - fixed_cash_in;
    let fee = amount - net;
    (net, fee)
}

/// Fee calculation for reversal (Go: fixedRefCashout, percentRefCashout).
fn calculate_fee_reversal(amount: f64, fixed: f64, percent: f64) -> (f64, f64) {
    let markup = 1.0 - percent / 100.0;
    let net = amount * markup - fixed;
    let rate = amount - net;
    (net, rate)
}

pub struct PixWebhookServiceImpl {
    transaction_repo: Arc<dyn TransactionRepository>,
    key_pix_repo: Arc<dyn KeyPixRepository>,
    account_rules_repo: Arc<dyn AccountRulesRepository>,
    with_list_accounts_repo: Arc<dyn WithListAccountsRepository>,
    fees_repo: Arc<dyn FeesRepository>,
    accounts_repo: Arc<dyn AccountsRepository>,
    provider_selector: Arc<dyn ProviderSelector>,
    invoice_repo: Arc<dyn InvoiceRepository>,
    sec_med_repo: Arc<dyn SecMedRepository>,
    batch_processor: Option<Arc<WebhookBatchProcessor>>,
    gateway_recorder: Option<Arc<dyn GatewayRecorder>>,
    default_partners_id: i64,
}

impl PixWebhookServiceImpl {
    pub fn new(
        transaction_repo: Arc<dyn TransactionRepository>,
        key_pix_repo: Arc<dyn KeyPixRepository>,
        account_rules_repo: Arc<dyn AccountRulesRepository>,
        with_list_accounts_repo: Arc<dyn WithListAccountsRepository>,
        fees_repo: Arc<dyn FeesRepository>,
        accounts_repo: Arc<dyn AccountsRepository>,
        provider_selector: Arc<dyn ProviderSelector>,
        invoice_repo: Arc<dyn InvoiceRepository>,
        sec_med_repo: Arc<dyn SecMedRepository>,
        default_partners_id: i64,
    ) -> Self {
        Self {
            transaction_repo,
            key_pix_repo,
            account_rules_repo,
            with_list_accounts_repo,
            fees_repo,
            accounts_repo,
            provider_selector,
            invoice_repo,
            sec_med_repo,
            batch_processor: None,
            gateway_recorder: None,
            default_partners_id,
        }
    }

    pub fn with_batch_processor(mut self, bp: Arc<WebhookBatchProcessor>) -> Self {
        self.batch_processor = Some(bp);
        self
    }

    pub fn with_gateway_recorder(mut self, gr: Arc<dyn GatewayRecorder>) -> Self {
        self.gateway_recorder = Some(gr);
        self
    }
}

#[async_trait]
impl PixWebhookService for PixWebhookServiceImpl {
    async fn receive_pix_in(&self, req: ReceivePixInRequest) -> Result<ReceivePixInResponse, Box<dyn std::error::Error + Send + Sync>> {
        if req.amount <= 0.0 {
            return Err("amount must be greater than 0".into());
        }
        if req.pix_key.is_empty() {
            return Err("pixKey is required".into());
        }
        if req.end_to_end_id.is_empty() {
            return Err("endToEndId is required".into());
        }

        let idempotency_key = if req.idempotency_key.is_empty() {
            req.end_to_end_id.clone()
        } else {
            req.idempotency_key.clone()
        };

        // 1. Idempotency check
        if let Some(existing_id) = self
            .transaction_repo
            .find_pix_in_duplicate(&req.end_to_end_id, &idempotency_key)
            .await
            .map_err(|e| e.to_string())?
        {
            return Ok(ReceivePixInResponse {
                status_code: 200,
                transaction_id: existing_id.to_string(),
                message: "PIX IN already processed (idempotent)".to_string(),
            });
        }

        // 2. Get account by PIX key
        let key_pix = self
            .key_pix_repo
            .get_by_key(&req.pix_key)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("account not found for PIX key: {}", req.pix_key))?;

        let account_id = key_pix.account_id;

        // 3. Validate account_rules (receive_external)
        let receive_external = self
            .account_rules_repo
            .get_by_account_id(account_id)
            .await
            .map_err(|e| e.to_string())?
            .map(|r| r.receive_external)
            .unwrap_or(true);

        if !receive_external {
            let is_whitelisted = self
                .with_list_accounts_repo
                .is_whitelisted_for_pix_in(account_id)
                .await
                .map_err(|e| e.to_string())?;
            if !is_whitelisted {
                return Err("account is not allowed to receive external PIX and is not whitelisted".into());
            }
        }

        // 4. Get provider (taxas customer ou admin)
        let provider = self
            .provider_selector
            .get_provider_for_account(account_id)
            .await
            .map_err(|e| e.to_string())?;

        // PIX IN limit advisory: Banco Central imposes higher default limits for PJ (type 2).
        // PF accounts that receive unusually large amounts are flagged for review (not blocked).
        const PIX_LIMIT_PF: f64 = 20_000.0;  // R$ 20.000 — standard daily advisory for PF
        const PIX_LIMIT_PJ: f64 = 500_000.0; // R$ 500.000 — advisory cap for PJ per transaction
        let person_type_id = provider.customer.person_type_id;
        let limit = if person_type_id == 2 { PIX_LIMIT_PJ } else { PIX_LIMIT_PF };
        if req.amount > limit {
            tracing::warn!(
                account_id = account_id,
                person_type_id = person_type_id,
                amount = req.amount,
                limit = limit,
                "pix_in: amount exceeds advisory limit for person type — flagging for review"
            );
        }

        let fixed = if provider.customer.fixed_cash_in > 0.0 {
            provider.customer.fixed_cash_in
        } else {
            provider.admin.fixed_cash_in
        };
        let percent = if provider.customer.percent_cashin > 0.0 {
            provider.customer.percent_cashin
        } else {
            provider.admin.percent_cashin
        };

        let markup = 1.0 - percent / 100.0;
        let net_amount_f64 = req.amount * markup - fixed;
        let fee_total_f64 = req.amount - net_amount_f64;
        let fee_percent_amount_f64 = req.amount * (percent / 100.0);
        let amount_dec = Decimal::try_from(req.amount).map_err(|e| e.to_string())?;

        // 5. Gateway: invoice (QR Code) ou payload
        // Para o simulador, aceitamos explicitamente `isQRCodePayment=true` para marcar como QR Code
        // sem depender de invoice já cadastrada.
        let (gateway, pix_operation_type, invoice_id, invoice_type_id) = if req.is_qr_code_payment {
            let gw = if req.gateway.is_empty() {
                String::new()
            } else {
                normalize_gateway(&req.gateway)
            };
            (gw, "pix_in_qrcode".to_string(), 0i64, 0i64)
        } else if let Some((inv_id, inv_type_id, _pl_id, gw_desc)) = self
            .invoice_repo
            .get_by_external_id(&req.end_to_end_id)
            .await
            .map_err(|e| e.to_string())?
        {
            let gw = gw_desc
                .map(|s| normalize_gateway(&s))
                .unwrap_or_else(|| normalize_gateway(&req.gateway));
            (gw, "pix_in_qrcode".to_string(), inv_id, inv_type_id)
        } else {
            let gw = if req.gateway.is_empty() {
                String::new()
            } else {
                normalize_gateway(&req.gateway)
            };
            (gw, "pix_in_key".to_string(), 0i64, 0i64)
        };

        let partners_id = if provider.admin.partners.id > 0 {
            provider.admin.partners.id
        } else {
            self.default_partners_id
        };

        // 6. Create PIX IN credit transaction (with fee audit fields)
        let requested_dec = Decimal::try_from(req.amount).map_err(|e| e.to_string())?;
        let net_dec = Decimal::try_from(net_amount_f64).map_err(|e| e.to_string())?;
        let total_dec = Decimal::try_from(req.amount).map_err(|e| e.to_string())?;
        let fee_fixed_dec = Decimal::try_from(fixed).map_err(|e| e.to_string())?;
        let fee_pct_rate_dec = Decimal::try_from(percent).map_err(|e| e.to_string())?;
        let fee_pct_amt_dec = Decimal::try_from(fee_percent_amount_f64).map_err(|e| e.to_string())?;
        let fee_total_dec = Decimal::try_from(fee_total_f64).map_err(|e| e.to_string())?;
        let partner_fix_dec = Decimal::try_from(provider.admin.fixed_cash_in).map_err(|e| e.to_string())?;
        let partner_pct_dec = Decimal::try_from(provider.admin.percent_cashin).map_err(|e| e.to_string())?;

        let transaction_id = self
            .transaction_repo
            .insert_pix_in_credit(
                account_id,
                invoice_id,
                partners_id,
                &req.end_to_end_id,
                &capitalize_initial(&req.payer_name),
                &extract_numbers(&req.payer_document),
                amount_dec,
                &req.pix_key,
                &req.description,
                &gateway,
                &pix_operation_type,
                requested_dec,
                net_dec,
                total_dec,
                fee_fixed_dec,
                fee_pct_rate_dec,
                fee_pct_amt_dec,
                fee_total_dec,
                partner_fix_dec,
                partner_pct_dec,
            )
            .await
            .map_err(|e| e.to_string())?;

        // 7. TTO (Taxa Operacional) se fee_total > 0
        if fee_total_f64 > 0.0 {
            let fee_dec = Decimal::try_from(fee_total_f64).map_err(|e| e.to_string())?;
            let endtoend = generate_end_to_end();
            // TTO DEBIT na conta do customer
            let tto_debit_id = self
                .transaction_repo
                .insert_tto(
                    account_id,
                    transaction_id,
                    &endtoend,
                    &provider.admin.data.full_name,
                    &provider.admin.data.document_number,
                    fee_dec,
                    1, // DEBIT
                    "Debit for operational transaction fee.",
                )
                .await
                .map_err(|e| e.to_string())?;
            // TTO CREDIT na conta do admin
            let endtoend2 = generate_end_to_end();
            let tto_credit_id = self
                .transaction_repo
                .insert_tto(
                    provider.admin.account_id,
                    tto_debit_id,
                    &endtoend2,
                    &provider.customer.data.full_name,
                    &provider.customer.data.document_number,
                    fee_dec,
                    2, // CREDIT
                    "Credit for operational transaction fee.",
                )
                .await
                .map_err(|e| e.to_string())?;

            // 8. TPO (Taxa Parceiro) se admin tem taxas e partners.id > 0
            if provider.admin.partners.id > 0
                && (provider.admin.fixed_cash_in > 0.0 || provider.admin.percent_cashin > 0.0)
            {
                let markup_partner = 1.0 - provider.admin.percent_cashin / 100.0;
                let calc_partner = req.amount * markup_partner - provider.admin.fixed_cash_in;
                let rate_partner = (req.amount - calc_partner).max(0.0);
                if rate_partner > 0.0 {
                    let rate_dec = Decimal::try_from(rate_partner).map_err(|e| e.to_string())?;
                    let endtoend3 = generate_end_to_end();
                    self.transaction_repo
                        .insert_tpo(
                            provider.admin.account_id,
                            provider.admin.partners.id,
                            tto_credit_id,
                            &endtoend3,
                            &provider.admin.partners.name,
                            rate_dec,
                            "Debit for operational Parteners rate",
                        )
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        // 9. MED (Reserva de Segurança) se percent_sec_med > 0
        let mut _sec_med_id: i64 = 0;
        if provider.customer.percent_sec_med > 0.0 {
            let percent_med = provider.customer.percent_sec_med;
            let markup_med = 1.0 - percent_med / 100.0;
            let amount_med = req.amount * markup_med;
            let rate_med = req.amount - amount_med;
            if rate_med > 0.0 {
                let rate_med_dec = Decimal::try_from(rate_med).map_err(|e| e.to_string())?;
                let endtoend_med = generate_end_to_end();
                let smd_id = self
                    .transaction_repo
                    .insert_smd(
                        account_id,
                        transaction_id,
                        &endtoend_med,
                        &provider.admin.data.full_name,
                        &provider.admin.data.document_number,
                        rate_med_dec,
                        "Debit for operational med security.",
                    )
                    .await
                    .map_err(|e| e.to_string())?;

                let scheduled_date = Utc::now() + chrono::Duration::days(90);
                let sec_med = SecMed {
                    id: 0,
                    account_id,
                    invoice_id: if invoice_id > 0 { invoice_id } else { 0 },
                    partners_id: if provider.admin.partners.id > 0 {
                        provider.admin.partners.id
                    } else {
                        0
                    },
                    apagar: String::new(),
                    transaction_id: smd_id,
                    status_sec_med_id: 1, // OPEN
                    amount: rate_med_dec,
                    scheduled_date: Some(scheduled_date),
                    deleted_at: None,
                    full_count: None,
                };
                _sec_med_id = self.sec_med_repo.insert(&sec_med).await.map_err(|e| e.to_string())?;
            }
        }

        // 10. Atualizar invoice status (DONE=2) se existir e não for FIXED (3)
        if invoice_id > 0 && invoice_type_id != 3 {
            let _ = self
                .invoice_repo
                .update_status(invoice_id, 2)
                .await
                .map_err(|e| e.to_string());
        }

        Ok(ReceivePixInResponse {
            status_code: 200,
            transaction_id: transaction_id.to_string(),
            message: "PIX IN received and credited successfully".to_string(),
        })
    }

    async fn receive_pix_out(&self, req: ReceivePixOutRequest) -> Result<ReceivePixOutResponse, Box<dyn std::error::Error + Send + Sync>> {
        if req.status.is_empty() {
            return Err("status is required".into());
        }
        if req.internal_transaction_id.is_empty() && req.end_to_end_id.is_empty() {
            return Err("either endToEndId or internalTransactionId is required".into());
        }

        let transaction_id = if !req.internal_transaction_id.is_empty() {
            let parsed = req.internal_transaction_id
                .parse::<i64>()
                .map_err(|_| format!("invalid internalTransactionId: {}", req.internal_transaction_id))?;
            if parsed <= 0 {
                return Err(format!("invalid internalTransactionId: {}", req.internal_transaction_id).into());
            }
            parsed
        } else {
            self.transaction_repo
                .find_id_by_external_id(&req.end_to_end_id)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("transaction not found for endToEndId: {}", req.end_to_end_id))?
        };

        let normalized_gateway = normalize_gateway(&req.gateway_name);

        // Map status: completed/success -> 4, failed/error -> 7, else -> 3 AWAITING
        let (new_status, msg_error) = match req.status.to_lowercase().as_str() {
            "completed" | "success" | "successful" => (4i64, String::new()),
            "failed" | "error" | "rejected" => {
                let msg = if req.message.is_empty() {
                    format!("Gateway error: {}", req.error_code)
                } else {
                    req.message.clone()
                };
                (7, msg)
            }
            _ => (3, String::new()), // processing, pending, in_progress -> AWAITING
        };

        if let Some(ref bp) = self.batch_processor {
            bp.queue_update(transaction_id, new_status, &normalized_gateway, &msg_error);
        } else {
            self.transaction_repo
                .update_pix_status(transaction_id, new_status, &msg_error, &normalized_gateway)
                .await
                .map_err(|e| format!("failed to update transaction status: {}", e))?;
        }

        if let Some(ref gr) = self.gateway_recorder {
            let gr = Arc::clone(gr);
            let gw = normalized_gateway.clone();
            let err = msg_error.clone();
            tokio::spawn(async move {
                if new_status == 4 {
                    gr.record_success(&gw).await;
                } else if new_status == 7 {
                    gr.record_error(&gw, 0, transaction_id, "webhook_failed", &err).await;
                }
            });
        }

        let status_msg = match new_status {
            4 => "completed",
            7 => "failed",
            _ => "awaiting",
        };

        Ok(ReceivePixOutResponse {
            status_code: 200,
            transaction_id: transaction_id.to_string(),
            message: format!("Transaction status updated to {}", status_msg),
        })
    }

    async fn send_reversal(&self, user_id: i64, req: SendReversalRequest) -> Result<SendReversalResponse, Box<dyn std::error::Error + Send + Sync>> {
        if req.amount <= 0.0 {
            return Err("error in amount".into());
        }
        if req.end2end.is_empty() {
            return Err("end2end is required".into());
        }

        // 1. Get account by user_id
        let account = self
            .accounts_repo
            .get_by_authentication_id(user_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("account not found for user {}", user_id))?;

        if account.account_status_id == 4 {
            return Err("your account is pending kyc".into());
        }
        if account.account_status_id != 1 {
            return Err("invalid account status".into());
        }

        let account_id = account.id;

        // 2. Get fees for reversal
        let (fixed, percent) = self
            .fees_repo
            .get_by_account_id(account_id)
            .await
            .map_err(|e| e.to_string())?
            .map(|f| {
                let fix = f.fixed_ref_cash_out.to_string().parse::<f64>().unwrap_or(0.0);
                let pct = f.percent_ref_cashout.to_string().parse::<f64>().unwrap_or(0.0);
                (fix, pct)
            })
            .unwrap_or((0.0, 0.0));

        let (_net, rate) = calculate_fee_reversal(req.amount, fixed, percent);

        // 3. Idempotency (external_id)
        if !req.external_id.is_empty() {
            if self
                .transaction_repo
                .find_reversal_duplicate(&req.external_id, account_id)
                .await
                .map_err(|e| e.to_string())?
                .is_some()
            {
                return Err(format!("transaction {} is duplicate", req.external_id).into());
            }
        }

        // 4. Balance check
        let balance = self
            .transaction_repo
            .get_balance(account_id)
            .await
            .map_err(|e| e.to_string())?;
        let balance_f64: f64 = balance.to_string().parse().unwrap_or(0.0);
        if balance_f64 + rate <= req.amount {
            return Err(format!("insufficient balance, your balance is {:.2}", balance_f64).into());
        }

        // 5. Find original transaction
        let (original_id, _orig_account, original_amount) = self
            .transaction_repo
            .find_original_for_reversal(&req.end2end)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("original transaction not found for endtoend {}", req.end2end))?;

        let original_amount_f64: f64 = original_amount.to_string().parse().unwrap_or(0.0);
        if req.amount > original_amount_f64 {
            return Err("error in amount".into());
        }

        // 6. Create reversal transaction (DPIX: sub_type=2)
        let external_id = if req.external_id.is_empty() {
            format!("rev_{}_{}", req.end2end, chrono::Utc::now().timestamp_millis())
        } else {
            req.external_id.clone()
        };

        let amount_dec = Decimal::try_from(req.amount).map_err(|e| e.to_string())?;
        let reversal_tx = Transaction {
            id: 0,
            account_id,
            invoice_id: 0,
            partners_id: self.default_partners_id,
            transaction_id: String::new(),
            charger_back_id: original_id.to_string(),
            parent_id: 0,
            external_id: external_id.clone(),
            name: String::new(),
            email: String::new(),
            document_number: String::new(),
            description: format!("Reversal for end2end {}", req.end2end),
            phone: String::new(),
            amount: amount_dec,
            isbp: String::new(),
            bank_name: String::new(),
            branch: String::new(),
            account: String::new(),
            endtoend_id: String::new(),
            pix_key_type_id: 0,
            key: String::new(),
            type_transaction_id: 2, // CREDIT (reversal = devolver valor)
            sub_type_transaction_id: 2, // DPIX
            remittance_information: String::new(),
            status_transaction_id: 4, // COMPLETED
            msg_error: String::new(),
            telegram_notification: false,
            try_count: 0,
            deleted_at: None,
            endtoend_id_temp: String::new(),
            full_count: None,
        };

        let reversal_id = self.transaction_repo.insert(&reversal_tx).await.map_err(|e| e.to_string())?;

        let mut data = std::collections::HashMap::new();
        data.insert("end2end".to_string(), serde_json::Value::String(req.end2end));
        data.insert("amount".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(req.amount).unwrap_or(serde_json::Number::from(0))));

        Ok(SendReversalResponse {
            status_code: 200,
            transaction_id: reversal_id.to_string(),
            data,
        })
    }
}
