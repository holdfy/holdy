//! Consulta e mint de saldo BRLx da carteira testnet usada pelos pedidos locais
//! (`holdfy-buyer`), via `stellar` CLI + identidade `holdfy-deployer` (issuer).
//! Só habilitado fora de mainnet — mesmo gate do `dev_handler`.

use axum::{extract::State, Json};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::Deserialize;
use tokio::process::Command;

use crate::error::AdminError;
use crate::state::AdminState;

const STROOPS_PER_BRLX: i64 = 10_000_000;

fn network() -> String {
    std::env::var("APICASH_STELLAR_NETWORK").unwrap_or_else(|_| "testnet".to_string())
}

fn dev_enabled() -> bool {
    network().trim().to_lowercase() != "mainnet"
}

fn stellar_bin() -> String {
    std::env::var("APICASH_STELLAR_CLI_BIN").unwrap_or_else(|_| "stellar".to_string())
}

fn rpc_url() -> String {
    std::env::var("APICASH_SOROBAN_RPC_URL")
        .or_else(|_| std::env::var("SOROBAN_RPC_URL"))
        .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string())
}

fn passphrase() -> String {
    std::env::var("APICASH_STELLAR_NETWORK_PASSPHRASE")
        .or_else(|_| std::env::var("STELLAR_NETWORK_PASSPHRASE"))
        .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string())
}

fn token_contract_id() -> Result<String, AdminError> {
    std::env::var("APICASH_BRLX_TOKEN_CONTRACT_ID")
        .map_err(|_| AdminError::internal("APICASH_BRLX_TOKEN_CONTRACT_ID não configurado"))
}

fn buyer_identity() -> String {
    std::env::var("APICASH_DEV_BUYER_IDENTITY").unwrap_or_else(|_| "holdfy-buyer".to_string())
}

fn deployer_identity() -> String {
    std::env::var("APICASH_DEV_DEPLOYER_IDENTITY").unwrap_or_else(|_| "holdfy-deployer".to_string())
}

async fn keys_address(identity: &str) -> Result<String, AdminError> {
    let out = Command::new(stellar_bin())
        .args(["keys", "address", identity])
        .output()
        .await
        .map_err(|e| AdminError::internal(format!("stellar keys address falhou: {e}")))?;
    if !out.status.success() {
        return Err(AdminError::internal(format!(
            "stellar keys address {identity}: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

async fn query_balance(token_contract: &str, address: &str) -> Result<i128, AdminError> {
    let out = Command::new(stellar_bin())
        .args([
            "contract",
            "invoke",
            "--id",
            token_contract,
            "--source",
            &deployer_identity(),
            "--network-passphrase",
            &passphrase(),
            "--rpc-url",
            &rpc_url(),
            "--",
            "balance",
            "--id",
            address,
        ])
        .output()
        .await
        .map_err(|e| AdminError::internal(format!("stellar balance falhou: {e}")))?;
    if !out.status.success() {
        return Err(AdminError::internal(format!(
            "stellar balance: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let trimmed = text.trim().trim_matches('"');
    trimmed
        .parse::<i128>()
        .map_err(|_| AdminError::internal(format!("balance inesperado: {trimmed}")))
}

pub async fn get_wallet(
    State(_state): State<AdminState>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if !dev_enabled() {
        return Err(AdminError::BadRequest(
            "dev wallet desabilitado: APICASH_STELLAR_NETWORK=mainnet".to_string(),
        ));
    }
    let token = token_contract_id()?;
    let buyer = keys_address(&buyer_identity()).await?;
    let deployer = keys_address(&deployer_identity()).await?;
    let balance_stroops = query_balance(&token, &buyer).await?;
    let balance_brlx = Decimal::from_i128_with_scale(balance_stroops, 7);

    Ok(Json(serde_json::json!({
        "network": network(),
        "token_contract_id": token,
        "buyer_address": buyer,
        "deployer_address": deployer,
        "balance_stroops": balance_stroops.to_string(),
        "balance_brlx": balance_brlx.to_string(),
    })))
}

#[derive(Debug, Deserialize)]
pub struct MintBody {
    pub amount: Decimal,
}

pub async fn mint_wallet(
    State(_state): State<AdminState>,
    Json(body): Json<MintBody>,
) -> Result<Json<serde_json::Value>, AdminError> {
    if !dev_enabled() {
        return Err(AdminError::BadRequest(
            "dev wallet desabilitado: APICASH_STELLAR_NETWORK=mainnet".to_string(),
        ));
    }
    if body.amount <= Decimal::ZERO {
        return Err(AdminError::BadRequest("amount deve ser positivo".to_string()));
    }

    let token = token_contract_id()?;
    let buyer = keys_address(&buyer_identity()).await?;

    let stroops_dec = (body.amount * Decimal::from(STROOPS_PER_BRLX)).trunc();
    let stroops = stroops_dec
        .to_i128()
        .filter(|v| *v > 0)
        .ok_or_else(|| AdminError::BadRequest("amount inválido".to_string()))?;

    let out = Command::new(stellar_bin())
        .args([
            "contract",
            "invoke",
            "--id",
            &token,
            "--source",
            &deployer_identity(),
            "--send=yes",
            "--network-passphrase",
            &passphrase(),
            "--rpc-url",
            &rpc_url(),
            "--",
            "mint",
            "--to",
            &buyer,
            "--amount",
            &stroops.to_string(),
        ])
        .output()
        .await
        .map_err(|e| AdminError::internal(format!("stellar mint falhou: {e}")))?;

    if !out.status.success() {
        return Err(AdminError::internal(format!(
            "stellar mint: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }

    let new_balance = query_balance(&token, &buyer).await?;
    let new_balance_brlx = Decimal::from_i128_with_scale(new_balance, 7);

    Ok(Json(serde_json::json!({
        "minted_stroops": stroops.to_string(),
        "buyer_address": buyer,
        "balance_stroops": new_balance.to_string(),
        "balance_brlx": new_balance_brlx.to_string(),
    })))
}
