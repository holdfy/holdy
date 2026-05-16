//! Direct **Horizon** REST access for ledger reads (transactions, accounts).
//!
//! Soroban simulation / contract calls will use [`stellar_rpc_client`] separately when feature `soroban` is enabled.

use reqwest::Client;
use serde_json::Value;
use tracing::instrument;

use crate::errors::AnchorError;
use crate::models::StellarTransaction;

/// Thin Horizon client (JSON parsing kept defensive for evolving Horizon schemas).
pub struct StellarClient {
    http: Client,
    horizon_base: String,
}

impl StellarClient {
    pub fn new(http: Client, horizon_base: String) -> Self {
        Self { http, horizon_base }
    }

    /// `GET /transactions/{tx_id}` — maps core fields into [`StellarTransaction`].
    #[instrument(skip(self), fields(tx_id = tx_id))]
    pub async fn get_transaction(&self, tx_id: &str) -> Result<StellarTransaction, AnchorError> {
        let url = format!(
            "{}/transactions/{}",
            self.horizon_base.trim_end_matches('/'),
            tx_id
        );
        let resp = self.http.get(&url).send().await?.error_for_status()?;
        let v: Value = resp.json().await?;
        parse_horizon_transaction(&v)
    }
}

fn parse_horizon_transaction(v: &Value) -> Result<StellarTransaction, AnchorError> {
    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let from = v
        .get("source_account")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let memo = v
        .get("memo")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    // Horizon does not flatten amount/asset per operation here — placeholder for first payment op if present.
    let (amount, asset, to) = extract_first_payment_like(v);
    Ok(StellarTransaction {
        id,
        amount,
        asset,
        from,
        to,
        memo,
    })
}

fn extract_first_payment_like(v: &Value) -> (String, String, String) {
    if let Some(ops) = v
        .get("_embedded")
        .and_then(|e| e.get("operations"))
        .and_then(|o| o.as_array())
    {
        for op in ops {
            if op.get("type").and_then(|t| t.as_str()) == Some("payment") {
                let amt = op
                    .get("amount")
                    .and_then(|a| a.as_str())
                    .unwrap_or("0")
                    .to_string();
                let asset = op
                    .get("asset_type")
                    .and_then(|a| a.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let to = op
                    .get("to")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                return (amt, asset, to);
            }
        }
    }
    ("0".into(), "native".into(), String::new())
}
