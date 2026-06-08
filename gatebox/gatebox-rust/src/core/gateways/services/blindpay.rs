// BlindPay HTTP service — PIX ↔ Stellar stablecoin anchor
// Docs: https://www.blindpay.com/docs
// Auth: Bearer API key (no OAuth — get_token_out returns the key as token).
// PIX IN:  payin-quote + payin/stellar  → returns PIX copia-e-cola
// PIX OUT: payout-quote + authorize/stellar + sign XDR + payout/evm
use async_trait::async_trait;
use reqwest::Client;

use crate::core::gateways::interfaces::blindpay::{
    BlindPayAuthorizeStellarRequest, BlindPayAuthorizeStellarResponse,
    BlindPayPayinQuoteRequest, BlindPayPayinQuoteResponse,
    BlindPayPayinRequest, BlindPayPayinResponse,
    BlindPayPayoutQuoteRequest, BlindPayPayoutQuoteResponse,
    BlindPayPayoutRequest, BlindPayPayoutResponse,
    BlindPayWalletResponse,
};

fn token() -> String {
    // USDB em dev (testnet), USDC em produção
    std::env::var("BLINDPAY_TOKEN").unwrap_or_else(|_| "USDB".to_string())
}

fn blockchain() -> String {
    // stellar_testnet em dev, stellar em prod
    match network().as_str() {
        "testnet" => "stellar_testnet".to_string(),
        _ => "stellar".to_string(),
    }
}
use crate::core::gateways::interfaces::sulcred::{
    SulcredSendPixKeyRequest, SulcredSendPixKeyResponse,
    SulcredSendPixKeyData, SulcredSendPixKeyDataPayment,
    SulcredSendPixKeyAccount, SulcredSendPixKeyPaymentResponse,
};
use crate::core::gateways::services::traits::{
    AuthOutResponse, BalanceResponse, CreateDynamicQrcodeRequest, CreateDynamicQrcodeResponse,
    GatewayHttpService, SendPixKeyResponse,
};
use crate::core::rabbitmq::GatewayFailureConfig;

fn base_url() -> String {
    std::env::var("BLINDPAY_BASE_URL")
        .unwrap_or_else(|_| "https://api.blindpay.com".to_string())
}

fn instance_id() -> String {
    std::env::var("BLINDPAY_INSTANCE_ID").unwrap_or_default()
}

fn api_key() -> String {
    std::env::var("BLINDPAY_API_KEY").unwrap_or_default()
}

fn wallet_id() -> String {
    std::env::var("BLINDPAY_WALLET_ID").unwrap_or_default()
}

fn stellar_secret() -> Option<String> {
    std::env::var("BLINDPAY_STELLAR_SECRET").ok().filter(|s| !s.is_empty())
}

fn network() -> String {
    std::env::var("BLINDPAY_STELLAR_NETWORK")
        .unwrap_or_else(|_| "testnet".to_string())
}

pub struct BlindPayHttpService {
    client: Client,
    base_url: String,
    instance_id: String,
}

impl Default for BlindPayHttpService {
    fn default() -> Self {
        Self {
            client: Client::new(),
            base_url: base_url(),
            instance_id: instance_id(),
        }
    }
}

impl BlindPayHttpService {
    fn url(&self, path: &str) -> String {
        format!(
            "{}/v1/instances/{}/{}",
            self.base_url.trim_end_matches('/'),
            self.instance_id,
            path.trim_start_matches('/')
        )
    }

    /// Sign a Stellar XDR transaction using the `stellar` CLI.
    /// Returns the signed XDR string.
    async fn sign_xdr(&self, xdr: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let secret = stellar_secret().ok_or("BLINDPAY_STELLAR_SECRET not set — cannot sign XDR for payout")?;
        let net = network();
        let output = tokio::process::Command::new("stellar")
            .args(["tx", "sign", "--sign-with-key", &secret, "--network", &net, xdr])
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("stellar CLI not found: {}", e))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("stellar tx sign failed: {}", err).into());
        }
        let signed = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(signed)
    }
}

#[async_trait]
impl GatewayHttpService for BlindPayHttpService {
    /// BlindPay uses Bearer API key — no OAuth needed.
    /// Returns the API key as the access_token so the rest of the pipeline works unchanged.
    async fn get_token_out(
        &self,
        _client_id: &str,
        _client_secret: &str,
    ) -> Result<AuthOutResponse, Box<dyn std::error::Error + Send + Sync>> {
        Ok(AuthOutResponse {
            access_token: api_key(),
            expires_in: 86400,
            token_type: Some("Bearer".to_string()),
        })
    }

    /// PIX OUT: stablecoin → PIX ao destinatário.
    /// Fluxo: payout-quote → authorize/stellar (XDR) → sign → payout.
    async fn send_pix_key(
        &self,
        _token: &str,
        request: &SulcredSendPixKeyRequest,
        _failure_config: Option<&GatewayFailureConfig>,
    ) -> Result<SendPixKeyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let key = api_key();
        let customer_id = std::env::var("BLINDPAY_PAYOUT_CUSTOMER_ID").unwrap_or_default();
        let bank_account_id = std::env::var("BLINDPAY_PAYOUT_BANK_ACCOUNT_ID").unwrap_or_default();

        if customer_id.is_empty() || bank_account_id.is_empty() {
            return Err("BLINDPAY_PAYOUT_CUSTOMER_ID and BLINDPAY_PAYOUT_BANK_ACCOUNT_ID are required for PIX OUT".into());
        }

        // 1. Create payout quote
        let quote_req = BlindPayPayoutQuoteRequest {
            customer_id: customer_id.clone(),
            bank_account_id: bank_account_id.clone(),
            request_amount: request.payment.amount * 100.0, // centavos
            cover_fees: true,
            blockchain: blockchain(),
            token: token(),
        };
        let quote_res = self
            .client
            .post(self.url("quotes"))
            .bearer_auth(&key)
            .json(&quote_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay payout-quote request: {}", e))?;
        if !quote_res.status().is_success() {
            let status = quote_res.status();
            let text = quote_res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("blindpay payout-quote HTTP {}: {}", status, text).into());
        }
        let quote: BlindPayPayoutQuoteResponse = quote_res
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay payout-quote json: {}", e))?;

        // 2. Authorize Stellar (get XDR)
        let auth_req = BlindPayAuthorizeStellarRequest {
            quote_id: quote.id.clone(),
            blockchain_wallet_id: wallet_id(),
        };
        let auth_res = self
            .client
            .post(self.url("authorize/stellar"))
            .bearer_auth(&key)
            .json(&auth_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay authorize/stellar request: {}", e))?;
        if !auth_res.status().is_success() {
            let status = auth_res.status();
            let text = auth_res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("blindpay authorize/stellar HTTP {}: {}", status, text).into());
        }
        let auth: BlindPayAuthorizeStellarResponse = auth_res
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay authorize/stellar json: {}", e))?;

        // 3. Sign XDR
        let signed_xdr = self.sign_xdr(&auth.xdr).await?;

        // 4. Execute payout
        let payout_req = BlindPayPayoutRequest {
            quote_id: quote.id.clone(),
            signed_xdr,
        };
        let payout_res = self
            .client
            .post(self.url("payouts/evm"))
            .bearer_auth(&key)
            .json(&payout_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay payout request: {}", e))?;
        if !payout_res.status().is_success() {
            let status = payout_res.status();
            let text = payout_res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("blindpay payout HTTP {}: {}", status, text).into());
        }
        let payout: BlindPayPayoutResponse = payout_res
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay payout json: {}", e))?;

        let end_to_end_id = payout
            .end_to_end_id
            .clone()
            .unwrap_or_else(|| format!("BP{}", payout.id));
        let status_str = payout.status.clone().unwrap_or_else(|| "PROCESSING".to_string());
        let amount_f64 = quote.fiat_amount.unwrap_or(request.payment.amount);
        let amount_str = format!("{:.2}", amount_f64);
        let now = chrono::Utc::now().to_rfc3339();
        let empty_account = SulcredSendPixKeyAccount {
            ispb: String::new(),
            document: String::new(),
            name: String::new(),
            number: String::new(),
            issuer: String::new(),
            account_type: String::new(),
        };

        Ok(SulcredSendPixKeyResponse {
            end_to_end_id: end_to_end_id.clone(),
            event_date: now.clone(),
            status: status_str.clone(),
            id: 0,
            payment: SulcredSendPixKeyPaymentResponse {
                currency: "BRL".to_string(),
                amount: amount_str,
            },
            type_: "PIX".to_string(),
            data: SulcredSendPixKeyData {
                id: 0,
                refunds: vec![],
                idempotency_key: payout.id.clone(),
                end_to_end_id,
                pix_key: request.pix_key.clone(),
                payment: SulcredSendPixKeyDataPayment {
                    currency: "BRL".to_string(),
                    amount: amount_f64,
                },
                status: status_str,
                transaction_type: "PIX".to_string(),
                local_instrument: "DICT".to_string(),
                created_at: now,
                creditor_account: empty_account.clone(),
                debtor_account: empty_account,
                remittance_information: request.description.clone(),
                error_code: String::new(),
                tx_id: payout.id,
                credit_debit_type: "CREDIT".to_string(),
            },
        })
    }

    /// Account balance from our BlindPay blockchain wallet.
    async fn get_balance(
        &self,
        _token: &str,
    ) -> Result<BalanceResponse, Box<dyn std::error::Error + Send + Sync>> {
        let wid = wallet_id();
        if wid.is_empty() {
            return Ok(BalanceResponse { balance: None, available: None });
        }
        let key = api_key();
        let url = self.url(&format!("blockchain-wallets/{}", wid));
        let res = self
            .client
            .get(&url)
            .bearer_auth(&key)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay balance request: {}", e))?;
        if !res.status().is_success() {
            return Ok(BalanceResponse { balance: None, available: None });
        }
        let raw: BlindPayWalletResponse = res
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay balance json: {}", e))?;
        let balance = raw.balance.as_ref().and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
        let available = raw.available_balance.as_ref().and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).or(balance);
        Ok(BalanceResponse { balance, available })
    }

    /// PIX IN: generate dynamic PIX QR code.
    /// Fluxo: payin-quote → payin/stellar → retorna copia-e-cola.
    async fn create_dynamic_qrcode(
        &self,
        _token: &str,
        request: &CreateDynamicQrcodeRequest,
    ) -> Result<CreateDynamicQrcodeResponse, Box<dyn std::error::Error + Send + Sync>> {
        let key = api_key();
        let wid = wallet_id();

        if wid.is_empty() {
            return Err("BLINDPAY_WALLET_ID is required for PIX IN (create_dynamic_qrcode)".into());
        }

        // 1. Create payin quote
        let quote_req = BlindPayPayinQuoteRequest {
            blockchain_wallet_id: wid,
            currency_type: "receiver".to_string(),
            request_amount: (request.amount * 100.0).round(), // centavos, min 500
            payment_method: "pix".to_string(), // lowercase obrigatório
            token: token(),
        };
        let quote_res = self
            .client
            .post(self.url("payin-quotes"))
            .bearer_auth(&key)
            .json(&quote_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay payin-quote request: {}", e))?;
        if !quote_res.status().is_success() {
            let status = quote_res.status();
            let text = quote_res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("blindpay payin-quote HTTP {}: {}", status, text).into());
        }
        let quote: BlindPayPayinQuoteResponse = quote_res
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay payin-quote json: {}", e))?;

        // 2. Create payin (generates PIX code)
        let customer_id = std::env::var("BLINDPAY_PAYIN_CUSTOMER_ID").ok().filter(|s| !s.is_empty());
        let payin_req = BlindPayPayinRequest {
            payin_quote_id: quote.id.clone(),
            customer_id,
        };
        let payin_res = self
            .client
            .post(self.url("payins/stellar"))
            .bearer_auth(&key)
            .json(&payin_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay payin request: {}", e))?;
        if !payin_res.status().is_success() {
            let status = payin_res.status();
            let text = payin_res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("blindpay payin HTTP {}: {}", status, text).into());
        }
        let payin: BlindPayPayinResponse = payin_res
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("blindpay payin json: {}", e))?;

        let pix_code = payin.pix_code
            .or(payin.pix_qr_code)
            .ok_or_else(|| anyhow::anyhow!("blindpay payin response has no pix_code/brcode field — verifique ToS do customer no dashboard BlindPay"))?;

        Ok(CreateDynamicQrcodeResponse {
            pix_copia_e_cola: pix_code,
            txid: payin.txid.or(Some(payin.id)),
            location: None,
            status: payin.status,
        })
    }
}
