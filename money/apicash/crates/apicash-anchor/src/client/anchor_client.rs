//! HTTP client for **Anchor** on/off-ramp endpoints (PIX ↔ **BRLx**).
//!
//! Real anchors implement SEP-6/SEP-24; paths below are conventional placeholders — adjust per provider.

use apicash_shared::Money;
use reqwest::Client;
use serde_json::json;
use tracing::instrument;

use crate::errors::AnchorError;
use crate::models::{OffRampResponse, OnRampResponse};

/// Calls Anchor APIs that coordinate fiat rails vs Stellar credits / debits.
pub struct AnchorClient {
    http: Client,
    anchor_base: String,
    asset_code: String,
}

impl AnchorClient {
    pub fn new(http: Client, anchor_base: String, asset_code: String) -> Self {
        Self {
            http,
            anchor_base,
            asset_code,
        }
    }

    /// Request on-ramp: PIX in → token credit (BRLx) — body shape is illustrative.
    #[instrument(skip(self))]
    pub async fn request_deposit_pix(
        &self,
        amount: Money,
        memo: &str,
    ) -> Result<OnRampResponse, AnchorError> {
        let url = format!("{}/v1/pix/deposit", self.anchor_base.trim_end_matches('/'));
        let body = json!({
            "asset": self.asset_code,
            "amount": amount.decimal().to_string(),
            "memo": memo,
        });
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let v: serde_json::Value = resp.json().await?;
        Ok(OnRampResponse {
            transaction_id: v
                .get("transaction_id")
                .and_then(|x| x.as_str())
                .map(ToOwned::to_owned),
            external_id: v
                .get("external_id")
                .and_then(|x| x.as_str())
                .map(ToOwned::to_owned),
            fiat_rail: "anchor".to_string(),
            stellar_tx_hash: v
                .get("stellar_tx_hash")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            status: v
                .get("status")
                .and_then(|x| x.as_str())
                .unwrap_or("pending")
                .to_string(),
            pix_br_code: v
                .get("pix_br_code")
                .and_then(|x| x.as_str())
                .map(ToOwned::to_owned),
            gateway: v
                .get("gateway")
                .and_then(|x| x.as_str())
                .map(ToOwned::to_owned),
            estimated_completion: chrono::Utc::now() + chrono::Duration::minutes(15),
        })
    }

    /// Request off-ramp: token burn / debit → PIX out.
    #[instrument(skip(self))]
    pub async fn request_withdraw_pix(
        &self,
        token_amount: Money,
        destination_pix_key: &str,
    ) -> Result<OffRampResponse, AnchorError> {
        let url = format!("{}/v1/pix/withdraw", self.anchor_base.trim_end_matches('/'));
        let body = json!({
            "asset": self.asset_code,
            "amount": token_amount.decimal().to_string(),
            "pix_key": destination_pix_key,
        });
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let v: serde_json::Value = resp.json().await?;
        let received = v
            .get("received_pix")
            .and_then(|x| x.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| token_amount.decimal());
        Ok(OffRampResponse {
            transaction_id: v
                .get("transaction_id")
                .and_then(|x| x.as_str())
                .map(ToOwned::to_owned),
            external_id: v
                .get("external_id")
                .and_then(|x| x.as_str())
                .map(ToOwned::to_owned),
            tx_hash: v
                .get("tx_hash")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            status: v
                .get("status")
                .and_then(|x| x.as_str())
                .unwrap_or("pending")
                .to_string(),
            gateway: v
                .get("gateway")
                .and_then(|x| x.as_str())
                .map(ToOwned::to_owned),
            received_pix: received,
        })
    }

    /// Query funding transaction status in the provider rail.
    #[instrument(skip(self))]
    pub async fn get_pix_transaction(
        &self,
        transaction_id: &str,
    ) -> Result<serde_json::Value, AnchorError> {
        let url = format!(
            "{}/v1/pix/transaction/{}",
            self.anchor_base.trim_end_matches('/'),
            transaction_id
        );
        let resp = self.http.get(&url).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }
}
