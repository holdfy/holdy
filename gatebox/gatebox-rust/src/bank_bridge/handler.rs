use std::sync::{Arc, OnceLock};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use regex::Regex;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{info, warn};

use crate::bank_bridge::whatsapp_notify;
use crate::transaction::TransactionRepository;

/// Bearer esperado pelo banco (`GATEBOX_API_KEY` no `backend_banco`). Sobrescrever com `GATEBOX_BANK_BRIDGE_API_KEY`.
#[derive(Clone)]
pub struct BankBridgeState {
    pub api_key: String,
    pub tx_repo: Arc<dyn TransactionRepository>,
}

#[derive(Debug, Deserialize)]
struct ValidateBody {
    reference: String,
}

#[derive(Debug, Serialize)]
struct ChargeValidationResponse {
    valid: bool,
    charge_id: String,
    amount_cents: i64,
    receiver: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    failure_message: String,
}

#[derive(Debug, Deserialize)]
struct NotifyBody {
    payment_id: String,
    charge_id: String,
    status: String,
}

fn bearer_ok(headers: &HeaderMap, expected: &str) -> bool {
    let Some(auth) = headers.get(axum::http::header::AUTHORIZATION) else {
        return false;
    };
    let Ok(s) = auth.to_str() else {
        return false;
    };
    let Some(token) = s.strip_prefix("Bearer ") else {
        return false;
    };
    token == expected
}

fn money_to_cents(amount: Decimal) -> i64 {
    let s = amount.round_dp(2).to_string();
    let parts: Vec<&str> = s.split('.').collect();
    let whole: i64 = parts.first().and_then(|x| x.parse().ok()).unwrap_or(0);
    let frac: i64 = parts
        .get(1)
        .map(|x| {
            let two: String = x.chars().take(2).collect();
            let padded = format!("{two:0<2}");
            padded.parse().unwrap_or(0i64)
        })
        .unwrap_or(0);
    whole.saturating_mul(100).saturating_add(frac)
}

/// QR sintético devolvido por `PixPrincipalServiceStub` / `synthetic_pix_qrcode_response`
/// (`GATEBOXRUST:QR:TX…:10.00`). Não há linha em `transaction` até integração completa de cobrança.
fn parse_gateboxrust_synthetic_qr(reference: &str) -> Option<(i64, String)> {
    let r = reference.trim();
    // Novo: GATEBOXRUST:QR|order_{uuid}|13.00
    if let Some(rest) = r.strip_prefix("GATEBOXRUST:QR|") {
        let (ref_token, amount_str) = rest.rsplit_once('|')?;
        let amount_f: f64 = amount_str.trim().parse().ok()?;
        if amount_f < 0.01 || amount_f > 100_000_000.0 {
            return None;
        }
        let cents = ((amount_f * 100.0).round() as i64).clamp(1, 2_147_483_647);
        let charge_id = ref_token.trim().to_string();
        if charge_id.is_empty() {
            return None;
        }
        return Some((cents, charge_id));
    }
    // Legado: GATEBOXRUST:QR:TX…:13.00
    const PREFIX: &str = "GATEBOXRUST:QR:";
    if !r.starts_with(PREFIX) {
        return None;
    }
    let rest = r.strip_prefix(PREFIX)?;
    let parts: Vec<&str> = rest.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let tx_token = parts[0].trim();
    let amount_f: f64 = parts[1].trim().parse().ok()?;
    if amount_f < 0.01 || amount_f > 100_000_000.0 {
        return None;
    }
    let cents = ((amount_f * 100.0).round() as i64).clamp(1, 2_147_483_647);
    let charge_id = if tx_token.is_empty() {
        format!("chg_rust_{}", hex::encode(&Sha256::digest(r.as_bytes())[..8]))
    } else {
        tx_token.to_string()
    };
    Some((cents, charge_id))
}

/// BR Code PIX EMV (copia-e-cola); alinhado ao stub em `backend_banco` (`pix_emv_stub`).
fn parse_emv_br_code_amount_cents(reference: &str) -> Option<i64> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"54(\d{2})(\d+\.\d{2})").expect("emv tag54 regex"));
    let caps = re.captures(reference)?;
    let amount_str = caps.get(2)?.as_str();
    let f: f64 = amount_str.parse().ok()?;
    if f < 0.01 || f > 100_000_000.0 {
        return None;
    }
    Some(((f * 100.0).round() as i64).clamp(1, 2_147_483_647))
}

fn emv_stub_charge_id(reference: &str) -> String {
    let mut h = Sha256::new();
    h.update(reference.as_bytes());
    format!("sandbox-emv-{}", hex::encode(&h.finalize()[..10]))
}

async fn resolve_tx(
    repo: &Arc<dyn TransactionRepository>,
    reference: &str,
) -> Result<Option<crate::model::Transaction>, crate::transaction::RepositoryError> {
    let r = reference.trim();
    if r.is_empty() {
        return Ok(None);
    }
    if let Some(id) = repo.find_id_by_external_id(r).await? {
        return repo.get_by_id(id).await;
    }
    if let Ok(id) = r.parse::<i64>() {
        return repo.get_by_id(id).await;
    }
    Ok(None)
}

async fn validate_charge(
    State(state): State<BankBridgeState>,
    headers: HeaderMap,
    Json(body): Json<ValidateBody>,
) -> (StatusCode, Json<ChargeValidationResponse>) {
    if !bearer_ok(&headers, &state.api_key) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ChargeValidationResponse {
                valid: false,
                charge_id: String::new(),
                amount_cents: 0,
                receiver: String::new(),
                failure_message: "unauthorized".into(),
            }),
        );
    }

    let r = body.reference.trim();
    if r.starts_with("sandbox:") {
        let parts: Vec<&str> = r.split(':').collect();
        let cents = parts.get(1).and_then(|s| s.parse::<i64>().ok()).unwrap_or(1000);
        let tail = parts.get(2).copied().unwrap_or("demo");
        return (
            StatusCode::OK,
            Json(ChargeValidationResponse {
                valid: true,
                charge_id: format!("chg_{tail}"),
                amount_cents: cents,
                receiver: "gatebox-sandbox".into(),
                failure_message: String::new(),
            }),
        );
    }

    match resolve_tx(&state.tx_repo, r).await {
        Ok(Some(tx)) => {
            let cents = money_to_cents(tx.amount);
            let charge_id = format!("{}", tx.id);
            let receiver = if !tx.key.is_empty() {
                tx.key.clone()
            } else if !tx.name.is_empty() {
                tx.name.clone()
            } else {
                "gatebox".into()
            };
            (
                StatusCode::OK,
                Json(ChargeValidationResponse {
                    valid: true,
                    charge_id,
                    amount_cents: cents,
                    receiver,
                    failure_message: String::new(),
                }),
            )
        }
        Ok(None) => {
            if let Some((amount_cents, charge_id)) = parse_gateboxrust_synthetic_qr(r) {
                return (
                    StatusCode::OK,
                    Json(ChargeValidationResponse {
                        valid: true,
                        charge_id,
                        amount_cents,
                        receiver: "gatebox-synthetic-qr".into(),
                        failure_message: String::new(),
                    }),
                );
            }
            if r.len() >= 32 && r.starts_with("000201") {
                if let Some(amount_cents) = parse_emv_br_code_amount_cents(r) {
                    let charge_id = emv_stub_charge_id(r);
                    return (
                        StatusCode::OK,
                        Json(ChargeValidationResponse {
                            valid: true,
                            charge_id,
                            amount_cents,
                            receiver: "PIX_BR_CODE_EMV".into(),
                            failure_message: String::new(),
                        }),
                    );
                }
            }
            (
                StatusCode::OK,
                Json(ChargeValidationResponse {
                    valid: false,
                    charge_id: String::new(),
                    amount_cents: 0,
                    receiver: String::new(),
                    failure_message: "charge not found for reference".into(),
                }),
            )
        }
        Err(e) => {
            warn!(error = %e, "bank_bridge validate: db error");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChargeValidationResponse {
                    valid: false,
                    charge_id: String::new(),
                    amount_cents: 0,
                    receiver: String::new(),
                    failure_message: "internal error".into(),
                }),
            )
        }
    }
}

async fn notify_status(
    State(state): State<BankBridgeState>,
    headers: HeaderMap,
    Json(body): Json<NotifyBody>,
) -> StatusCode {
    if !bearer_ok(&headers, &state.api_key) {
        return StatusCode::UNAUTHORIZED;
    }
    info!(
        payment_id = %body.payment_id,
        charge_id = %body.charge_id,
        status = %body.status,
        "bank_bridge: external bank payment status (integração Banco Saczuck)"
    );

    let st = body.status.to_ascii_uppercase();
    let mark_paid = matches!(st.as_str(), "APPROVED" | "COMPLETED" | "PAID");
    if mark_paid && !body.charge_id.is_empty() {
        if let Ok(id) = body.charge_id.parse::<i64>() {
            if let Err(e) = state
                .tx_repo
                .update_pix_status(id, 4, "approved_via_banco_saczuck", "banco_saczuck")
                .await
            {
                warn!(error = %e, tx_id = id, "bank_bridge: notify could not mark numeric tx completed");
            } else {
                info!(tx_id = id, "bank_bridge: transaction marked completed (numeric charge_id)");
            }
        } else if let Ok(Some(id)) = state.tx_repo.find_id_by_external_id(&body.charge_id).await {
            if let Err(e) = state
                .tx_repo
                .update_pix_status(id, 4, "approved_via_banco_saczuck", "banco_saczuck")
                .await
            {
                warn!(error = %e, tx_id = id, external_id = %body.charge_id, "bank_bridge: notify could not mark tx completed");
            } else {
                info!(tx_id = id, external_id = %body.charge_id, "bank_bridge: transaction marked completed (external_id)");
            }
        }

        if let Some(reference) = whatsapp_notify::payment_reference_for_notify(&body.charge_id) {
            whatsapp_notify::spawn_whatsapp_payment_notify(reference);
        }
    }

    StatusCode::OK
}

pub fn routes(state: BankBridgeState) -> Router {
    Router::new()
        .route("/public/charges/validate", post(validate_charge))
        .route("/internal/bank/notify-status", post(notify_status))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gateboxrust_qr_parses_amount_and_charge_id() {
        let (cents, cid) =
            parse_gateboxrust_synthetic_qr("GATEBOXRUST:QR:TX999:12.34").expect("parse");
        assert_eq!(cents, 1234);
        assert_eq!(cid, "TX999");
    }

    #[test]
    fn gateboxrust_qr_pipe_format_with_order_ref() {
        let (cents, cid) = parse_gateboxrust_synthetic_qr(
            "GATEBOXRUST:QR|order_8e94736e-1f09-4d02-892c-5990fbfa4d70|13.00",
        )
        .expect("parse");
        assert_eq!(cents, 1300);
        assert_eq!(cid, "order_8e94736e-1f09-4d02-892c-5990fbfa4d70");
    }

    #[test]
    fn emv_br_extracts_tag54_amount() {
        let with_tag54 =
            "00020126580014br.gov.bcb.pix0136123456789012345678901234567890123456789012345678901234567890123456540510.505802BR5913";
        let cents = parse_emv_br_code_amount_cents(with_tag54).expect("amount");
        assert_eq!(cents, 1050);
    }
}
