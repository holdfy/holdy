//! Primary façade for **Stellar + Anchor**: this is the main contact surface with the Stellar network
//! for APICash. Fiat value is tokenized on-ledger as **BRLx** (or [`crate::config::StellarConfig::asset_code`]);
//! real BRL moves via PIX through the Anchor’s banking integration (`AnchorClient` HTTP API), unless
//! `APICASH_FIAT_RAIL=simulated|mock` com PIX EMV via **Gatebox** (`GATEBOX_BASE_URL`); sem Gatebox, use `anchor` + `APICASH_STELLAR_ANCHOR_URL`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use apicash_shared::Money;
use async_trait::async_trait;
use rust_decimal::prelude::ToPrimitive;
use tracing::instrument;

use crate::client::{fetch_dynamic_pix_qrcode, AnchorClient, StellarClient};
use crate::config::StellarConfig;
use crate::errors::AnchorError;
use crate::models::{
    EscrowTokenTransferResult, OffRampResponse, OnRampResponse, StellarTransaction,
};
use reqwest::Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiatRail {
    /// PIX deposit / settlement via configured anchor HTTP API (`APICASH_STELLAR_ANCHOR_URL`).
    Anchor,
    /// Desenvolvimento: respostas PIX determinísticas sem outbound HTTP (**nunca** usar em produção/compliance real).
    Simulated,
}

impl FiatRail {
    pub fn as_str(self) -> &'static str {
        match self {
            FiatRail::Anchor => "anchor",
            FiatRail::Simulated => "simulated",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettlementState {
    pub settled: bool,
    pub status: String,
    pub transaction_id: Option<String>,
}

#[async_trait]
pub trait FiatOnRamp: Send + Sync {
    fn rail(&self) -> FiatRail;
    async fn start_deposit(
        &self,
        amount: Money,
        memo: String,
    ) -> Result<OnRampResponse, AnchorError>;
    async fn withdraw_to_pix(
        &self,
        token_amount: Money,
        destination_pix_key: String,
        external_id: String,
        memo: String,
    ) -> Result<OffRampResponse, AnchorError>;
    async fn poll_settlement(
        &self,
        transaction_id: &str,
        external_id: Option<&str>,
    ) -> Result<SettlementState, AnchorError>;
}

/// Orchestrates Anchor (PIX / fiat rails) and Horizon (ledger reads).
pub struct AnchorService {
    #[allow(dead_code)]
    cfg: StellarConfig,
    stellar: StellarClient,
    fiat: Box<dyn FiatOnRamp>,
}

struct AnchorRail {
    anchor: AnchorClient,
}

#[async_trait]
impl FiatOnRamp for AnchorRail {
    fn rail(&self) -> FiatRail {
        FiatRail::Anchor
    }

    async fn start_deposit(
        &self,
        amount: Money,
        memo: String,
    ) -> Result<OnRampResponse, AnchorError> {
        let mut resp = self.anchor.request_deposit_pix(amount, &memo).await?;
        resp.fiat_rail = FiatRail::Anchor.as_str().to_string();
        if resp.external_id.is_none() {
            resp.external_id = Some(memo);
        }
        Ok(resp)
    }

    async fn withdraw_to_pix(
        &self,
        token_amount: Money,
        destination_pix_key: String,
        _external_id: String,
        _memo: String,
    ) -> Result<OffRampResponse, AnchorError> {
        self.anchor
            .request_withdraw_pix(token_amount, &destination_pix_key)
            .await
    }

    async fn poll_settlement(
        &self,
        transaction_id: &str,
        _external_id: Option<&str>,
    ) -> Result<SettlementState, AnchorError> {
        let raw = self.anchor.get_pix_transaction(transaction_id).await?;
        let status = raw
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending")
            .to_string();
        let settled = matches!(status.as_str(), "completed" | "settled" | "success");
        Ok(SettlementState {
            settled,
            status,
            transaction_id: Some(transaction_id.to_string()),
        })
    }
}

fn stable_digest(parts: &[&str]) -> u64 {
    let mut h = DefaultHasher::new();
    for p in parts {
        p.hash(&mut h);
    }
    h.finish()
}

/// PIX EMV só via Gatebox (`POST /api/v1/pix/qrcode`). `APICASH_GATEBOX_ENABLED=0|false` desliga explicitamente.
fn apicash_gatebox_pix_enabled() -> bool {
    let url_ok = std::env::var("GATEBOX_BASE_URL")
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if !url_ok {
        return false;
    }
    if let Ok(flag) = std::env::var("APICASH_GATEBOX_ENABLED") {
        let v = flag.trim().to_ascii_lowercase();
        if matches!(v.as_str(), "0" | "false" | "no" | "off") {
            return false;
        }
    }
    true
}

fn gatebox_pick_transaction_id(gb: &crate::client::GateboxQrCodeParsed, fallback_fp: u64) -> String {
    let a = gb.transaction_id.trim();
    if !a.is_empty() {
        return a.to_string();
    }
    let b = gb.tx_id.trim();
    if !b.is_empty() {
        return b.to_string();
    }
    format!("gatebox_dyn_{fallback_fp:016x}")
}

fn gatebox_pick_gateway_label(gb: &crate::client::GateboxQrCodeParsed) -> Option<String> {
    let g = gb.gateway.trim();
    Some(if g.is_empty() {
        "gatebox".to_string()
    } else {
        g.to_string()
    })
}

#[derive(Debug, Clone)]
struct SimulatedFiatRail {
    asset_code: String,
    http: Client,
}

#[async_trait]
impl FiatOnRamp for SimulatedFiatRail {
    fn rail(&self) -> FiatRail {
        FiatRail::Simulated
    }

    async fn start_deposit(
        &self,
        amount: Money,
        memo: String,
    ) -> Result<OnRampResponse, AnchorError> {
        if !apicash_gatebox_pix_enabled() {
            return Err(AnchorError::Config(
                "APICASH_FIAT_RAIL=simulated exige Gatebox: defina GATEBOX_BASE_URL em money/.env (apicash-core carrega via load_workspace_dotenv). Para desativar Gatebox use APICASH_FIAT_RAIL=anchor com APICASH_STELLAR_ANCHOR_URL."
                    .into(),
            ));
        }

        let amt = amount.decimal().to_string();
        let fp = stable_digest(&[memo.as_str(), &amt, &self.asset_code]);
        let stellar_tx_hash = format!("mock_stellar_sim_{}_{fp:016x}", amt.replace('.', "_"));

        let gb = fetch_dynamic_pix_qrcode(&self.http, amount, memo.as_str()).await?;
        tracing::info!(
            gateway = %gb.gateway,
            qr_len = %gb.qr_code.len(),
            "simulated fiat rail: PIX EMV from Gatebox (qr_code)"
        );
        let tx_id_gatebox = gatebox_pick_transaction_id(&gb, fp);
        let gw_lbl = gatebox_pick_gateway_label(&gb);
        let emv = gb.qr_code;
        Ok(OnRampResponse {
            transaction_id: Some(tx_id_gatebox),
            external_id: Some(memo),
            fiat_rail: FiatRail::Simulated.as_str().to_string(),
            stellar_tx_hash,
            status: "pending".to_string(),
            pix_br_code: Some(emv),
            gateway: gw_lbl,
            estimated_completion: chrono::Utc::now() + chrono::Duration::minutes(15),
        })
    }

    async fn withdraw_to_pix(
        &self,
        token_amount: Money,
        destination_pix_key: String,
        external_id: String,
        memo: String,
    ) -> Result<OffRampResponse, AnchorError> {
        let dec = token_amount.decimal().to_string();
        let fp = stable_digest(&[
            external_id.as_str(),
            memo.as_str(),
            dec.as_str(),
            destination_pix_key.as_str(),
        ]);
        Ok(OffRampResponse {
            transaction_id: Some(format!("sim_wd_{fp:016x}")),
            external_id: Some(external_id),
            tx_hash: format!("mock_pix_out_{fp:016x}"),
            status: "completed".to_string(),
            gateway: Some("simulated_local".into()),
            received_pix: token_amount.decimal(),
        })
    }

    async fn poll_settlement(
        &self,
        transaction_id: &str,
        _external_id: Option<&str>,
    ) -> Result<SettlementState, AnchorError> {
        Ok(SettlementState {
            settled: true,
            status: "completed".to_string(),
            transaction_id: Some(transaction_id.to_string()),
        })
    }
}

impl AnchorService {
    /// Build service from loaded config (operational [`StellarConfig::secret_key`] is held for future signing).
    pub fn new(cfg: StellarConfig) -> Self {
        let http = Client::new();
        let horizon = cfg.horizon_url.clone();
        let anchor_url = cfg.anchor_url.clone();
        let asset = cfg.asset_code.clone();
        let fiat = build_fiat_rail(http.clone(), anchor_url, asset);
        Self {
            cfg,
            stellar: StellarClient::new(http.clone(), horizon),
            fiat,
        }
    }

    /// On-ramp: PIX deposit instruction → Stellar credit of tokenized BRL (**BRLx**).
    #[instrument(skip(self, memo))]
    pub async fn deposit_pix(
        &self,
        amount: Money,
        memo: String,
    ) -> Result<OnRampResponse, AnchorError> {
        self.fiat.start_deposit(amount, memo).await
    }

    /// Off-ramp: redeem token balance → PIX payout.
    #[instrument(skip(self))]
    pub async fn withdraw_to_pix(
        &self,
        token_amount: Money,
        destination_pix_key: String,
        external_id: String,
        memo: String,
    ) -> Result<OffRampResponse, AnchorError> {
        self.fiat
            .withdraw_to_pix(token_amount, destination_pix_key, external_id, memo)
            .await
    }

    /// Poll funding settlement status in the configured fiat rail.
    #[instrument(skip(self))]
    pub async fn poll_funding_settlement(
        &self,
        transaction_id: &str,
        external_id: Option<&str>,
    ) -> Result<SettlementState, AnchorError> {
        self.fiat.poll_settlement(transaction_id, external_id).await
    }

    pub fn fiat_rail_name(&self) -> &'static str {
        self.fiat.rail().as_str()
    }

    /// Após on-ramp (PIX → BRLx), transfere BRLx para o endereço do contrato Soroban de escrow.
    #[instrument(skip(self, escrow_contract_address, memo))]
    pub async fn transfer_brlx_to_escrow(
        &self,
        escrow_contract_address: &str,
        amount: Money,
        memo: &str,
    ) -> Result<EscrowTokenTransferResult, AnchorError> {
        // Simulated PIX/Gatebox rail: mock off-chain transfer unless testnet on-chain is mandatory.
        if self.fiat.rail() == FiatRail::Simulated
            && !apicash_shared::require_testnet()
        {
            let amt_s = amount.decimal().to_string();
            let fp = stable_digest(&[escrow_contract_address, memo, amt_s.as_str()]);
            tracing::info!(
                %escrow_contract_address,
                amount = %amount,
                %memo,
                "simulated fiat rail: mock BRLx transfer to escrow (no stellar invoke)"
            );
            return Ok(EscrowTokenTransferResult {
                tx_hash: format!("mock_brlx_escrow_{fp:016x}"),
                status: "mock_transferred_to_escrow".into(),
                is_mock: true,
            });
        }

        let token_contract_id = std::env::var("APICASH_BRLX_TOKEN_CONTRACT_ID")
            .map_err(|_| AnchorError::Config("APICASH_BRLX_TOKEN_CONTRACT_ID missing".into()))?;
        let rpc = std::env::var("APICASH_SOROBAN_RPC_URL")
            .or_else(|_| std::env::var("SOROBAN_RPC_URL"))
            .map_err(|_| AnchorError::Config("APICASH_SOROBAN_RPC_URL missing".into()))?;
        let passphrase = std::env::var("APICASH_STELLAR_NETWORK_PASSPHRASE")
            .or_else(|_| std::env::var("STELLAR_NETWORK_PASSPHRASE"))
            .ok();
        let stellar_bin =
            std::env::var("APICASH_STELLAR_CLI_BIN").unwrap_or_else(|_| "stellar".into());
        let source_secret = std::env::var("APICASH_SOROBAN_BUYER_SOURCE")
            .or_else(|_| std::env::var("APICASH_STELLAR_BUYER_SOURCE"))
            .or_else(|_| std::env::var("APICASH_SOROBAN_SOURCE_SECRET"))
            .or_else(|_| std::env::var("SOROBAN_SOURCE_SECRET"))
            .map_err(|_| {
                AnchorError::Config(
                    "APICASH_SOROBAN_BUYER_SOURCE or APICASH_SOROBAN_SOURCE_SECRET missing".into(),
                )
            })?;
        let from_addr = std::env::var("APICASH_STELLAR_BUYER_ADDRESS")
            .map_err(|_| AnchorError::Config("APICASH_STELLAR_BUYER_ADDRESS missing".into()))?;

        let stroops = (amount.decimal() * rust_decimal::Decimal::from(10_000_000i64))
            .trunc()
            .to_i128()
            .unwrap_or(0);
        if stroops <= 0 {
            return Err(AnchorError::Validation("invalid amount".into()));
        }

        tracing::info!(
            %escrow_contract_address,
            %token_contract_id,
            %from_addr,
            amount = %amount,
            %memo,
            "live: token transfer BRLx -> escrow contract via stellar CLI"
        );

        let mut cmd = tokio::process::Command::new(&stellar_bin);
        cmd.args([
            "contract",
            "invoke",
            "--id",
            &token_contract_id,
            "--rpc-url",
            &rpc,
        ]);
        if let Some(ref p) = passphrase {
            cmd.args(["--network-passphrase", p]);
        }
        cmd.args([
            "--source",
            &source_secret,
            "--sign-with-key",
            &source_secret,
        ]);
        cmd.args([
            "--",
            "transfer",
            "--from",
            &from_addr,
            "--to",
            escrow_contract_address,
            "--amount",
            &stroops.to_string(),
        ]);

        let out = cmd
            .output()
            .await
            .map_err(|e| AnchorError::Anchor(e.to_string()))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(AnchorError::Anchor(format!(
                "stellar invoke transfer failed: {stderr}"
            )));
        }
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();

        Ok(EscrowTokenTransferResult {
            tx_hash: stdout,
            status: "transferred_to_escrow".into(),
            is_mock: false,
        })
    }

    /// Poll ledger / internal status by Stellar transaction id (hash).
    #[instrument(skip(self))]
    pub async fn get_transaction_status(
        &self,
        tx_id: &str,
    ) -> Result<StellarTransaction, AnchorError> {
        self.stellar.get_transaction(tx_id).await
    }
}

fn build_fiat_rail(http: Client, anchor_url: String, asset: String) -> Box<dyn FiatOnRamp> {
    let rail = std::env::var("APICASH_FIAT_RAIL")
        .unwrap_or_else(|_| "simulated".into())
        .to_ascii_lowercase();
    match rail.as_str() {
        "anchor" | "sep24" => Box::new(AnchorRail {
            anchor: AnchorClient::new(http, anchor_url, asset),
        }),
        "mock" | "simulated" => {
            tracing::warn!(
                APICASH_FIAT_RAIL = %rail,
                "fiat rail simulated: PIX só via Gatebox (GATEBOX_BASE_URL); para SEP use APICASH_FIAT_RAIL=anchor"
            );
            Box::new(SimulatedFiatRail {
                asset_code: asset,
                http,
            })
        }
        other => {
            panic!("invalid APICASH_FIAT_RAIL={other} (expected anchor, sep24, simulated, or mock)")
        }
    }
}
