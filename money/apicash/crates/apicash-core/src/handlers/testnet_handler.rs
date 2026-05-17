//! Recent Stellar testnet transactions recorded by APICash (from order persistence).

use std::sync::Arc;

use apicash_shared::utils::stellar::{default_horizon_url, parse_network_label, StellarNetworkKind};
use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::state::{AppState, StoredOrder};

#[derive(Debug, Deserialize)]
pub struct RecentTestnetQuery {
    /// Max items (default 10, cap 50).
    pub limit: Option<usize>,
    /// `db` (default): hashes guardados nos pedidos. `horizon`: contas do `.env`. `all`: união.
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentTestnetTx {
    pub hash: String,
    pub kind: String,
    pub order_id: uuid::Uuid,
    pub soroban_mode: String,
    pub recorded_at: DateTime<Utc>,
    pub explorer_url: String,
}

#[derive(Debug, Serialize)]
pub struct RecentTestnetResponse {
    pub network: String,
    pub horizon_url: String,
    pub limit: usize,
    pub count: usize,
    pub transactions: Vec<RecentTestnetTx>,
}

/// `GET /testnet/transactions` — last on-chain hashes produced by this APICash instance.
pub async fn recent_testnet_transactions(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RecentTestnetQuery>,
) -> Result<Json<RecentTestnetResponse>, (axum::http::StatusCode, String)> {
    let limit = q.limit.unwrap_or(10).clamp(1, 50);
    let network_label = std::env::var("APICASH_STELLAR_NETWORK")
        .or_else(|_| std::env::var("STELLAR_NETWORK"))
        .unwrap_or_else(|_| "testnet".into());
    let net_kind = parse_network_label(&network_label);
    let horizon_url = std::env::var("APICASH_STELLAR_HORIZON_URL")
        .or_else(|_| std::env::var("STELLAR_HORIZON_URL"))
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| default_horizon_url(net_kind).map(str::to_string))
        .unwrap_or_else(|| "https://horizon-testnet.stellar.org".into());
    let explorer_base = std::env::var("APICASH_STELLAR_EXPLORER_BASE").unwrap_or_else(|_| {
        match net_kind {
            StellarNetworkKind::Mainnet => "https://stellar.expert/explorer/public".into(),
            _ => "https://stellar.expert/explorer/testnet".into(),
        }
    });

    let source = q
        .source
        .as_deref()
        .unwrap_or("all")
        .to_ascii_lowercase();

    let mut rows: Vec<RecentTestnetTx> = Vec::new();

    if source == "db" || source == "all" {
        let orders = state
            .orders
            .list_all()
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;
        for stored in orders {
            push_tx(
                &mut rows,
                &stored,
                &explorer_base,
                "anchor",
                stored.anchor_tx_hash.as_deref(),
            );
            push_tx(
                &mut rows,
                &stored,
                &explorer_base,
                "brlx_escrow_transfer",
                stored.brlx_escrow_transfer_tx_hash.as_deref(),
            );
            push_tx(
                &mut rows,
                &stored,
                &explorer_base,
                "soroban_lock",
                stored.soroban_lock_tx_hash.as_deref(),
            );
            push_tx(
                &mut rows,
                &stored,
                &explorer_base,
                "off_ramp",
                stored.off_ramp_tx_hash.as_deref(),
            );
        }
    }

    if (source == "horizon" || source == "all") && rows.len() < limit {
        if let Err(e) = fetch_horizon_recent(&horizon_url, &explorer_base, limit, &mut rows).await
        {
            tracing::warn!(error = %e, "testnet/transactions: horizon fetch failed");
        }
    }

    rows.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));
    rows.dedup_by(|a, b| a.hash == b.hash);
    rows.truncate(limit);

    Ok(Json(RecentTestnetResponse {
        network: network_label,
        horizon_url,
        limit,
        count: rows.len(),
        transactions: rows,
    }))
}

fn push_tx(
    out: &mut Vec<RecentTestnetTx>,
    stored: &StoredOrder,
    explorer_base: &str,
    kind: &str,
    hash: Option<&str>,
) {
    let Some(hash) = hash.map(str::trim).filter(|h| !h.is_empty()) else {
        return;
    };
    if is_mock_hash(hash) {
        return;
    }
    out.push(RecentTestnetTx {
        explorer_url: format!("{explorer_base}/tx/{hash}"),
        hash: hash.to_string(),
        kind: kind.to_string(),
        order_id: stored.order.id,
        soroban_mode: stored.soroban_mode.clone(),
        recorded_at: stored.order.updated_at,
    });
}

async fn fetch_horizon_recent(
    horizon_url: &str,
    explorer_base: &str,
    limit: usize,
    out: &mut Vec<RecentTestnetTx>,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let accounts: Vec<(&str, String)> = [
        ("buyer", "APICASH_STELLAR_BUYER_ADDRESS"),
        ("seller", "APICASH_STELLAR_SELLER_ADDRESS"),
        ("escrow_contract", "APICASH_SOROBAN_ESCROW_CONTRACT_ID"),
        ("deployer", "APICASH_SOROBAN_ADMIN_ADDRESS"),
    ]
    .into_iter()
    .filter_map(|(kind, key)| {
        std::env::var(key)
            .ok()
            .filter(|s| !s.is_empty())
            .map(|addr| (kind, addr))
    })
    .collect();

    for (kind, account) in accounts {
        let url = format!(
            "{}/accounts/{account}/transactions?order=desc&limit={limit}",
            horizon_url.trim_end_matches('/'),
        );
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let records = body
            .get("_embedded")
            .and_then(|e| e.get("records"))
            .and_then(|r| r.as_array());
        let Some(records) = records else {
            continue;
        };
        for rec in records {
            let hash = rec
                .get("hash")
                .or_else(|| rec.get("id"))
                .and_then(|x| x.as_str())
                .unwrap_or("");
            if is_mock_hash(hash) {
                continue;
            }
            let created = rec
                .get("created_at")
                .and_then(|x| x.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);
            out.push(RecentTestnetTx {
                hash: hash.to_string(),
                kind: format!("horizon_{kind}"),
                order_id: uuid::Uuid::nil(),
                soroban_mode: "horizon".into(),
                recorded_at: created,
                explorer_url: format!("{explorer_base}/tx/{hash}"),
            });
        }
    }
    Ok(())
}

fn is_mock_hash(hash: &str) -> bool {
    let h = hash.to_ascii_lowercase();
    h.starts_with("mock_")
        || h.starts_with("mock_stellar")
        || h.contains("mock_brlx")
        || h == "mock_deploy_tx"
        || h.len() < 32
}
