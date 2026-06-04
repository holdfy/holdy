use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    gatebox,
    models::{
        DepositRequest, DepositResponse, HealthResponse, TransactionStatusResponse, WithdrawRequest,
        WithdrawResponse,
    },
    state::{AppState, TxKind, TxRecord},
};

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "simulator-anchor-pix",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// `POST /v1/pix/deposit` — on-ramp: cria intenção de depósito PIX.
///
/// 1. Obtém QR EMV do Gatebox (se disponível) ou gera fake local.
/// 2. Armazena registro em memória para polling de settlement.
/// 3. Retorna response compatível com `AnchorClient::request_deposit_pix`.
pub async fn deposit(
    State(state): State<AppState>,
    Json(req): Json<DepositRequest>,
) -> Result<Json<DepositResponse>, StatusCode> {
    let tx_id = format!("sim_anchor_{}", Uuid::new_v4().as_simple());
    let amount = req.amount.trim().to_string();
    let memo = req.memo.as_deref().unwrap_or("").to_string();

    if amount.is_empty() {
        warn!("deposit: amount ausente");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Tenta obter QR real do Gatebox; cai para fake se indisponível.
    let pix_br_code = if let Some(ref gb_url) = state.gatebox_base_url {
        match gatebox::fetch_qr_from_gatebox(
            &state.http,
            gb_url,
            state.gatebox_api_key.as_deref(),
            &amount,
            &memo,
        )
        .await
        {
            Some(qr) => {
                info!(tx_id = %tx_id, "deposit: QR EMV obtido via Gatebox");
                qr
            }
            None => {
                warn!(tx_id = %tx_id, "deposit: Gatebox indisponível — usando QR fake");
                gatebox::fake_pix_qr(&amount, &memo)
            }
        }
    } else {
        info!(tx_id = %tx_id, "deposit: gerando QR fake (GATEBOX_BASE_URL não configurado)");
        gatebox::fake_pix_qr(&amount, &memo)
    };

    let now = Utc::now();
    let estimated = now + chrono::Duration::milliseconds(state.auto_settle_ms as i64 + 5_000);

    // Guarda registro para o poll de settlement.
    {
        let mut txs = state.transactions.write().await;
        txs.insert(
            tx_id.clone(),
            TxRecord {
                id: tx_id.clone(),
                amount: amount.clone(),
                pix_key: None,
                kind: TxKind::OnRamp,
                created_at: now,
                pix_br_code: Some(pix_br_code.clone()),
            },
        );
    }

    info!(
        tx_id = %tx_id,
        amount = %amount,
        memo = %memo,
        settle_in_ms = state.auto_settle_ms,
        "📥 on-ramp: depósito PIX simulado criado",
    );

    Ok(Json(DepositResponse {
        transaction_id: tx_id.clone(),
        external_id: if memo.is_empty() { None } else { Some(memo) },
        status: "pending".to_string(),
        pix_br_code: Some(pix_br_code),
        stellar_tx_hash: format!("sim_pending_{}", &tx_id[..16]),
        estimated_completion: estimated.to_rfc3339(),
        gateway: "simulator-anchor-pix".to_string(),
        asset: state.asset_code.clone(),
        amount,
        simulated: true,
    }))
}

/// `GET /v1/pix/transaction/:id` — polling de settlement.
///
/// Retorna `"pending"` até `auto_settle_ms` após a criação, depois `"completed"`.
/// Compatível com `AnchorClient::get_pix_transaction` (lê apenas o campo `"status"`).
pub async fn transaction_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TransactionStatusResponse>, StatusCode> {
    let record = {
        let txs = state.transactions.read().await;
        txs.get(&id).cloned()
    };

    let Some(rec) = record else {
        warn!(tx_id = %id, "transaction_status: id não encontrado");
        return Err(StatusCode::NOT_FOUND);
    };

    let elapsed_ms = Utc::now()
        .signed_duration_since(rec.created_at)
        .num_milliseconds()
        .max(0) as u64;

    let status = if elapsed_ms >= state.auto_settle_ms {
        "completed"
    } else {
        "pending"
    };

    info!(
        tx_id = %id,
        elapsed_ms,
        settle_threshold_ms = state.auto_settle_ms,
        status,
        "⏱  polling settlement",
    );

    Ok(Json(TransactionStatusResponse {
        id,
        status: status.to_string(),
        amount: rec.amount,
        asset: state.asset_code.clone(),
    }))
}

/// `POST /v1/pix/withdraw` — off-ramp: simula envio de PIX ao vendedor.
///
/// Não faz nenhum pagamento real. Loga e retorna `"completed"` imediatamente.
/// Compatível com `AnchorClient::request_withdraw_pix` (lê `received_pix` como string).
pub async fn withdraw(
    State(state): State<AppState>,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, StatusCode> {
    let tx_id = format!("sim_wd_{}", Uuid::new_v4().as_simple());
    let amount = req.amount.trim().to_string();
    let pix_key = req.pix_key.trim().to_string();

    if amount.is_empty() || pix_key.is_empty() {
        warn!("withdraw: amount ou pix_key ausente");
        return Err(StatusCode::BAD_REQUEST);
    }

    {
        let mut txs = state.transactions.write().await;
        txs.insert(
            tx_id.clone(),
            TxRecord {
                id: tx_id.clone(),
                amount: amount.clone(),
                pix_key: Some(pix_key.clone()),
                kind: TxKind::OffRamp,
                created_at: Utc::now(),
                pix_br_code: None,
            },
        );
    }

    info!(
        tx_id = %tx_id,
        amount = %amount,
        pix_key = %pix_key,
        "💸 off-ramp: PIX simulado → vendedor",
    );

    Ok(Json(WithdrawResponse {
        transaction_id: tx_id.clone(),
        tx_hash: format!("sim_wd_hash_{}", &tx_id[..16]),
        status: "completed".to_string(),
        received_pix: amount,
        gateway: "simulator-anchor-pix".to_string(),
        pix_key,
        simulated: true,
    }))
}
