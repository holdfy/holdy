//! `GET /admin/stellar/transactions`
//! Lista pedidos com dados Stellar/Soroban: hashes on-chain, rede, identidade do comprador.

use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::Row;

use crate::error::AdminError;
use crate::state::AdminState;

#[derive(Debug, Serialize)]
pub struct StellarTxRow {
    pub order_id:                   String,
    pub buyer_name:                  String,
    pub buyer_document:              String,
    pub seller_id:                   String,
    pub amount_brl:                  String,
    pub order_status:                String,
    pub custody_status:              Option<String>,
    /// "real" | "mock" | "simulated"
    pub soroban_mode:                String,
    pub soroban_escrow_contract_id:  Option<String>,
    pub soroban_lock_tx_hash:        Option<String>,
    pub soroban_release_tx_hash:     Option<String>,
    pub brlx_transfer_tx_hash:       Option<String>,
    /// "testnet" | "mainnet" | "simulated"
    pub network:                     String,
    pub created_at:                  String,
    /// URL directa no Stellar Expert (None quando soroban_mode != "real")
    pub explorer_lock_url:           Option<String>,
    pub explorer_contract_url:       Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StellarTxListResponse {
    pub transactions: Vec<StellarTxRow>,
    pub total:        usize,
    pub network:      String,
}

pub async fn list_stellar_transactions(
    State(state): State<AdminState>,
) -> Result<Json<StellarTxListResponse>, AdminError> {
    let network = std::env::var("APICASH_STELLAR_NETWORK")
        .or_else(|_| std::env::var("STELLAR_NETWORK"))
        .unwrap_or_else(|_| "simulated".to_string());

    let explorer_base = match network.as_str() {
        "mainnet" => "https://stellar.expert/explorer/public",
        _         => "https://stellar.expert/explorer/testnet",
    };

    let pool = match state.pg_pool.as_ref() {
        Some(p) => p.clone(),
        None => {
            // Sem pool Postgres — devolve lista vazia (modo in-memory dev)
            return Ok(Json(StellarTxListResponse {
                transactions: vec![],
                total: 0,
                network,
            }));
        }
    };

    // Junta orders + custody + wa_contacts para obter nome e documento do comprador.
    let rows = sqlx::query(r#"
        SELECT
            o.id::text                          AS order_id,
            COALESCE(o.buyer_name, wc.name, '')  AS buyer_name,
            COALESCE(wc.document, '')             AS buyer_document,
            o.seller_id::text                    AS seller_id,
            o.amount::text                       AS amount_brl,
            o.status                             AS order_status,
            c.status                             AS custody_status,
            COALESCE(o.soroban_mode, 'simulated') AS soroban_mode,
            o.soroban_escrow_contract_id,
            o.soroban_lock_tx_hash,
            c.soroban_release_tx_hash,
            o.brlx_escrow_transfer_tx_hash,
            o.created_at
        FROM orders o
        LEFT JOIN custody c ON c.order_id = o.id
        LEFT JOIN wa_contacts wc ON wc.user_id = o.buyer_id
        WHERE o.soroban_mode IS NOT NULL
           OR o.soroban_lock_tx_hash IS NOT NULL
           OR o.fiat_rail = 'anchor'
        ORDER BY o.created_at DESC
        LIMIT 500
    "#)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("stellar_handler query failed: {e}");
        AdminError::internal(e.to_string())
    })?;

    let transactions: Vec<StellarTxRow> = rows.iter().map(|r| {
        let mode: String = r.try_get("soroban_mode").unwrap_or_else(|_| "simulated".into());
        let contract: Option<String> = r.try_get("soroban_escrow_contract_id").unwrap_or(None);
        let lock_hash: Option<String> = r.try_get("soroban_lock_tx_hash").unwrap_or(None);

        let explorer_lock_url = lock_hash.as_deref()
            .filter(|h| !h.starts_with("mock"))
            .map(|h| format!("{explorer_base}/tx/{h}"));

        let explorer_contract_url = contract.as_deref()
            .filter(|_| mode == "real")
            .map(|c| format!("{explorer_base}/contract/{c}"));

        let created_at: chrono::DateTime<chrono::Utc> = r
            .try_get("created_at")
            .unwrap_or_else(|_| chrono::Utc::now());

        StellarTxRow {
            order_id:                  r.try_get("order_id").unwrap_or_default(),
            buyer_name:                r.try_get("buyer_name").unwrap_or_default(),
            buyer_document:            r.try_get("buyer_document").unwrap_or_default(),
            seller_id:                 r.try_get("seller_id").unwrap_or_default(),
            amount_brl:                r.try_get("amount_brl").unwrap_or_default(),
            order_status:              r.try_get("order_status").unwrap_or_default(),
            custody_status:            r.try_get("custody_status").unwrap_or(None),
            soroban_mode:              mode,
            soroban_escrow_contract_id: contract,
            soroban_lock_tx_hash:      lock_hash,
            soroban_release_tx_hash:   r.try_get("soroban_release_tx_hash").unwrap_or(None),
            brlx_transfer_tx_hash:     r.try_get("brlx_escrow_transfer_tx_hash").unwrap_or(None),
            network:                   network.clone(),
            created_at:                created_at.to_rfc3339(),
            explorer_lock_url,
            explorer_contract_url,
        }
    }).collect();

    let total = transactions.len();
    Ok(Json(StellarTxListResponse { transactions, total, network }))
}
