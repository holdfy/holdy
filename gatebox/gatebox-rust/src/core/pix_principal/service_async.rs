// Async SendPix: validates account, creates transaction, publishes to queue.
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use rust_decimal::Decimal;

use crate::core::messaging::PaymentPublisher;
use crate::model::Transaction;
use crate::transaction::TransactionRepository;

use super::service::{PixPrincipalService, SendPixRequest, SendPixResponse};
use super::service::{synthetic_pix_qrcode_response, GenerateQrCodeRequest, GenerateQrCodeResponse};

/// Fee calculation result (Go: fixed, percent, markup, net_amount, rate, total_amount).
#[derive(Debug, Clone)]
pub struct FeeCalculation {
    pub requested_amount: f64,
    pub net_amount: f64,
    pub rate: f64,
    pub total_amount: f64,
}

fn calculate_fee(requested_amount: f64, fixed_cash_out: f64, percent_cashout: f64) -> FeeCalculation {
    let markup = 1.0 - percent_cashout / 100.0;
    let net_amount = requested_amount * markup - fixed_cash_out;
    let rate = requested_amount - net_amount;
    let total_amount = requested_amount + rate;
    FeeCalculation {
        requested_amount,
        net_amount,
        rate,
        total_amount,
    }
}

/// MED (Reserva de Segurança) amount for a PIX IN.
/// Returns the amount to be locked in sec_med given the raw PIX IN value and the configured %.
pub fn calculate_med_amount(pix_amount: f64, percent_sec_med: f64) -> f64 {
    if percent_sec_med <= 0.0 {
        return 0.0;
    }
    let markup = 1.0 - percent_sec_med / 100.0;
    let net = pix_amount * markup;
    pix_amount - net // == pix_amount * (percent_sec_med / 100)
}

/// Pure-Rust mirror of SQL_BALANCE invariant:
/// available = SUM(CREDIT, status∈{3,4}) − SUM(DEBIT, status∈{3,4}) − SUM(active_sec_med)
///
/// `credits` / `debits` are amounts (positive), `blocked_med` is total open sec_med.
pub fn compute_available_balance(credits: f64, debits: f64, blocked_med: f64) -> f64 {
    credits - debits - blocked_med
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- calculate_fee ----

    #[test]
    fn fee_no_fee() {
        let fc = calculate_fee(100.0, 0.0, 0.0);
        assert!((fc.net_amount - 100.0).abs() < 0.01);
        assert!((fc.rate - 0.0).abs() < 0.01);
        assert!((fc.total_amount - 100.0).abs() < 0.01);
    }

    #[test]
    fn fee_fixed_and_percent() {
        let fc = calculate_fee(100.0, 0.5, 1.0);
        assert!((fc.net_amount - 98.5).abs() < 0.01);
        assert!((fc.rate - 1.5).abs() < 0.01);
        assert!((fc.total_amount - 101.5).abs() < 0.01);
    }

    #[test]
    fn fee_only_percent() {
        let fc = calculate_fee(1000.0, 0.0, 2.5);
        // net = 1000 * 0.975 = 975; rate = 25; total = 1025
        assert!((fc.net_amount - 975.0).abs() < 0.01);
        assert!((fc.rate - 25.0).abs() < 0.01);
        assert!((fc.total_amount - 1025.0).abs() < 0.01);
    }

    #[test]
    fn fee_only_fixed() {
        let fc = calculate_fee(500.0, 3.0, 0.0);
        // net = 500 - 3 = 497; rate = 3; total = 503
        assert!((fc.net_amount - 497.0).abs() < 0.01);
        assert!((fc.rate - 3.0).abs() < 0.01);
        assert!((fc.total_amount - 503.0).abs() < 0.01);
    }

    #[test]
    fn fee_zero_amount() {
        let fc = calculate_fee(0.0, 0.5, 1.0);
        assert!(fc.net_amount < 0.0); // negative is valid input error — check doesn't blow up
    }

    // ---- MED (sec_med) calculation ----

    #[test]
    fn med_zero_percent() {
        assert!((calculate_med_amount(1000.0, 0.0)).abs() < 0.001);
    }

    #[test]
    fn med_negative_percent() {
        assert!((calculate_med_amount(1000.0, -5.0)).abs() < 0.001);
    }

    #[test]
    fn med_standard() {
        // 10% of 500 = 50
        let med = calculate_med_amount(500.0, 10.0);
        assert!((med - 50.0).abs() < 0.01);
    }

    #[test]
    fn med_small_percent() {
        // 0.5% of 10_000 = 50
        let med = calculate_med_amount(10_000.0, 0.5);
        assert!((med - 50.0).abs() < 0.01);
    }

    // ---- available balance invariant: CREDIT - DEBIT - MED ----

    #[test]
    fn balance_simple() {
        // PIX IN 1000, PIX OUT 200 → balance 800, no MED
        assert!((compute_available_balance(1000.0, 200.0, 0.0) - 800.0).abs() < 0.001);
    }

    #[test]
    fn balance_with_med() {
        // PIX IN 1000, PIX OUT 100, MED 50 → 850
        assert!((compute_available_balance(1000.0, 100.0, 50.0) - 850.0).abs() < 0.001);
    }

    #[test]
    fn balance_zero() {
        assert!((compute_available_balance(0.0, 0.0, 0.0)).abs() < 0.001);
    }

    #[test]
    fn balance_credits_equal_debits_plus_med() {
        // Full drain: 1000 CREDIT, 900 DEBIT, 100 MED → 0
        assert!((compute_available_balance(1000.0, 900.0, 100.0)).abs() < 0.001);
    }

    #[test]
    fn balance_multiple_credits_and_debits() {
        // 3 PIX IN: 500 + 300 + 200 = 1000; 2 PIX OUT: 150 + 250 = 400; MED: 80
        let credits = 500.0 + 300.0 + 200.0;
        let debits = 150.0 + 250.0;
        let med = 80.0;
        assert!((compute_available_balance(credits, debits, med) - 520.0).abs() < 0.001);
    }

    /// Verify the fee + MED together are consistent with net balance.
    #[test]
    fn fee_and_med_integration() {
        let pix_in = 1000.0;
        let med_pct = 10.0;
        let fee_fixed = 1.5;
        let fee_pct = 1.0;

        let med = calculate_med_amount(pix_in, med_pct);      // 100
        let fee = calculate_fee(pix_in, fee_fixed, fee_pct);  // net=988.5, rate=11.5
        // After PIX IN: CREDIT 1000, DEBIT fee.rate=11.5, MED=100 → available=888.5
        let available = compute_available_balance(pix_in, fee.rate, med);
        assert!((available - 888.5).abs() < 0.01);
    }
}

/// Async implementation: validates, creates tx, publishes to queue, returns immediately.
pub struct PixPrincipalServiceAsync {
    transaction_repo: Arc<dyn TransactionRepository>,
    accounts_repo: Arc<dyn crate::accounts::AccountsRepository>,
    account_rules_repo: Arc<dyn crate::account_rules::AccountRulesRepository>,
    with_list_accounts_repo: Arc<dyn crate::with_list_accounts::WithListAccountsRepository>,
    fees_repo: Arc<dyn crate::fees::FeesRepository>,
    publisher: Arc<dyn PaymentPublisher>,
    gateway_name: String,
    default_partners_id: i64,
}

impl PixPrincipalServiceAsync {
    pub fn new(
        transaction_repo: Arc<dyn TransactionRepository>,
        accounts_repo: Arc<dyn crate::accounts::AccountsRepository>,
        account_rules_repo: Arc<dyn crate::account_rules::AccountRulesRepository>,
        with_list_accounts_repo: Arc<dyn crate::with_list_accounts::WithListAccountsRepository>,
        fees_repo: Arc<dyn crate::fees::FeesRepository>,
        publisher: Arc<dyn PaymentPublisher>,
        gateway_name: String,
        default_partners_id: i64,
    ) -> Self {
        Self {
            transaction_repo,
            accounts_repo,
            account_rules_repo,
            with_list_accounts_repo,
            fees_repo,
            publisher,
            gateway_name,
            default_partners_id,
        }
    }
}

#[async_trait]
impl PixPrincipalService for PixPrincipalServiceAsync {
    async fn send_pix(&self, req: SendPixRequest) -> Result<SendPixResponse, Box<dyn std::error::Error + Send + Sync>> {
        let user_id = req.user_id.ok_or_else(|| anyhow::anyhow!("user_id required for async SendPix"))?;

        // 1. Resolve account from authentication_id
        let account = self
            .accounts_repo
            .get_by_authentication_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("account not found for user {}", user_id))?;

        if account.account_status_id != 1 {
            return Err(anyhow::anyhow!("invalid account status").into());
        }

        let account_id = account.id;

        // 2. account_rules: deposit_external (PIX OUT = saída externa)
        let deposit_external = self
            .account_rules_repo
            .get_by_account_id(account_id)
            .await?
            .map(|r| r.deposit_external)
            .unwrap_or(true); // Se não encontrar, assume permitido
        if !deposit_external {
            let is_whitelisted = self
                .with_list_accounts_repo
                .is_whitelisted_for_pix_out(account_id)
                .await?;
            if !is_whitelisted {
                return Err(anyhow::anyhow!(
                    "account is not allowed to deposit externally and is not whitelisted"
                )
                .into());
            }
        }

        let requested_amount = req.amount;
        if requested_amount <= 0.0 {
            return Err(anyhow::anyhow!("amount must be positive").into());
        }
        // 3. Fee calculation (from fees table or 0)
        let (fixed, percent) = self
            .fees_repo
            .get_by_account_id(account_id)
            .await?
            .map(|f| {
                let fix: f64 = f.fixed_cash_out.try_into().unwrap_or(0.0);
                let pct: f64 = f.percent_cashout.try_into().unwrap_or(0.0);
                (fix, pct)
            })
            .unwrap_or((0.0, 0.0));
        let fee_calc = calculate_fee(requested_amount, fixed, percent);
        let total_amount_dec = Decimal::try_from(fee_calc.total_amount).map_err(|e| anyhow::anyhow!("invalid total amount: {}", e))?;
        let net_amount_dec = Decimal::try_from(fee_calc.net_amount).map_err(|e| anyhow::anyhow!("invalid net amount: {}", e))?;

        // 4. Idempotency check (if external_id provided) - uses net_amount like Go
        let today = Utc::now().date_naive();
        let today_start = Utc.from_utc_datetime(&today.and_hms_opt(0, 0, 0).unwrap());
        let today_end = today_start + chrono::Duration::days(1);
        if let Some(ref ext_id) = req.external_id {
            if let Some(dup_id) = self
                .transaction_repo
                .find_duplicate_external_id(ext_id, account_id, net_amount_dec, today_start, today_end)
                .await?
            {
                return Err(anyhow::anyhow!("transaction {} is duplicate (id: {})", ext_id, dup_id).into());
            }
        }

        // 5. Balance check: balance >= total_amount (requested + fee)
        let balance = self.transaction_repo.get_balance(account_id).await?;
        let balance_f: f64 = balance.try_into().unwrap_or(0.0);
        if balance_f < fee_calc.total_amount {
            return Err(anyhow::anyhow!(
                "insufficient balance, your balance is {:.2} your rate {:.2} your withdraw {:.2}",
                balance_f,
                fee_calc.rate,
                requested_amount
            )
            .into());
        }

        // 6. Create transaction (status NEW, type DEBIT, sub_type PIX) - amount = total debit
        let tx = Transaction {
            id: 0,
            account_id,
            invoice_id: 0,
            partners_id: self.default_partners_id,
            transaction_id: String::new(),
            charger_back_id: String::new(),
            parent_id: 0,
            external_id: req.external_id.clone().unwrap_or_default(),
            name: req.name.clone(),
            email: String::new(),
            document_number: req.document_number.clone(),
            description: req.memo.clone().unwrap_or_default(),
            phone: String::new(),
            amount: total_amount_dec,
            isbp: req.bank.clone(),
            bank_name: req.bank.clone(),
            branch: req.branch.clone(),
            account: req.account.clone(),
            endtoend_id: String::new(),
            pix_key_type_id: 1,
            key: req.key.clone(),
            type_transaction_id: 1, // DEBIT (PIX OUT)
            sub_type_transaction_id: 1, // PIX
            remittance_information: req.memo.clone().unwrap_or_default(),
            status_transaction_id: 1, // NEW
            msg_error: String::new(),
            telegram_notification: false,
            try_count: 0,
            deleted_at: None,
            endtoend_id_temp: String::new(),
            full_count: None,
        };

        let payment_id = self.transaction_repo.insert(&tx).await?;

        // 7. Publish to queue - net_amount is what the gateway sends to recipient
        self.publisher
            .publish(payment_id, fee_calc.net_amount, None)
            .await
            .map_err(|e| anyhow::anyhow!("failed to publish to queue: {}", e))?;

        let mut data = std::collections::HashMap::new();
        data.insert("account".to_string(), serde_json::Value::String(req.account));
        data.insert("amount".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(fee_calc.net_amount).unwrap_or(serde_json::Number::from(0))));
        data.insert("rate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(fee_calc.rate).unwrap_or(serde_json::Number::from(0))));
        data.insert("key".to_string(), serde_json::Value::String(req.key));
        data.insert("name".to_string(), serde_json::Value::String(req.name));
        data.insert("status".to_string(), serde_json::Value::String("NEW".to_string()));
        data.insert("type".to_string(), serde_json::Value::String("DEBIT".to_string()));

        Ok(SendPixResponse {
            status_code: 200,
            transaction_id: payment_id.to_string(),
            data,
        })
    }

    async fn generate_qr_code(&self, req: GenerateQrCodeRequest) -> Result<GenerateQrCodeResponse, Box<dyn std::error::Error + Send + Sync>> {
        synthetic_pix_qrcode_response(req, &self.gateway_name)
    }
}
