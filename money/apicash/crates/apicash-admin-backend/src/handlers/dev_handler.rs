//! Ferramentas de desenvolvimento: forçar liquidação de pedidos `pending_funding`
//! sem PIX real, para agilizar testes locais/testnet. Nunca habilitado em mainnet.

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::error::AdminError;
use crate::state::AdminState;

fn dev_network() -> String {
    std::env::var("APICASH_STELLAR_NETWORK").unwrap_or_else(|_| "testnet".to_string())
}

fn dev_enabled() -> bool {
    dev_network().trim().to_lowercase() != "mainnet"
}

pub async fn dev_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "enabled": dev_enabled(),
        "network": dev_network(),
    }))
}

/// Chama `POST /internal/orders/{path}` no apicash-core com `{order_id}` no corpo,
/// autenticado por `APICASH_API_KEY` (nunca exposto ao browser).
async fn call_core_internal(path: &str, id: Uuid) -> Result<Json<serde_json::Value>, AdminError> {
    let core_url =
        std::env::var("APICASH_CORE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let api_key = std::env::var("APICASH_API_KEY").unwrap_or_default();

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{core_url}/internal/orders/{path}"))
        .header("x-api-key", &api_key)
        .json(&serde_json::json!({ "order_id": id }))
        .send()
        .await
        .map_err(|e| AdminError::internal(format!("apicash-core unreachable: {e}")))?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or_else(|_| serde_json::json!({}));

    if !status.is_success() {
        let msg = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("request failed")
            .to_string();
        return Err(AdminError::BadRequest(msg));
    }

    Ok(Json(body))
}

/// Chama `POST /internal/orders/settle` no apicash-core (mesmo mecanismo do poller de
/// funding automático) para pular a espera do PIX real em ambiente de teste.
/// `pending_funding` → `in_custody`.
pub async fn force_settle_order(
    State(_state): State<AdminState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if !dev_enabled() {
        return Err(AdminError::BadRequest(
            "force-settle desabilitado: APICASH_STELLAR_NETWORK=mainnet".to_string(),
        ));
    }
    call_core_internal("settle", id).await
}

/// Chama `POST /internal/orders/release` no apicash-core — equivalente ao comprador
/// confirmar entrega, sem exigir JWT. `in_custody` → `completed` (dispara off-ramp
/// automático se o vendedor tiver chave PIX cadastrada).
pub async fn force_release_order(
    State(_state): State<AdminState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if !dev_enabled() {
        return Err(AdminError::BadRequest(
            "force-release desabilitado: APICASH_STELLAR_NETWORK=mainnet".to_string(),
        ));
    }
    call_core_internal("release", id).await
}

#[derive(Debug, serde::Deserialize)]
pub struct ForceAcceptProposalBody {
    #[serde(default)]
    pub buyer_id: Option<Uuid>,
}

/// Chama `POST /internal/proposals/{id}/force-accept` no apicash-core — aceita a proposta
/// ignorando bloqueio anti-fraude (velocidade/volume/CPF). Destrava testes quando a política
/// de risco bloqueia legitimamente um comprador de teste.
pub async fn force_accept_proposal(
    State(_state): State<AdminState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ForceAcceptProposalBody>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if !dev_enabled() {
        return Err(AdminError::BadRequest(
            "force-accept desabilitado: APICASH_STELLAR_NETWORK=mainnet".to_string(),
        ));
    }
    let core_url =
        std::env::var("APICASH_CORE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let api_key = std::env::var("APICASH_API_KEY").unwrap_or_default();

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{core_url}/internal/proposals/{id}/force-accept"))
        .header("x-api-key", &api_key)
        .json(&serde_json::json!({ "buyer_id": body.buyer_id }))
        .send()
        .await
        .map_err(|e| AdminError::internal(format!("apicash-core unreachable: {e}")))?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap_or_else(|_| serde_json::json!({}));

    if !status.is_success() {
        let msg = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("request failed")
            .to_string();
        return Err(AdminError::BadRequest(msg));
    }

    Ok(Json(body))
}
