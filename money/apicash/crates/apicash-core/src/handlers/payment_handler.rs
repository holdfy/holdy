//! PIX payment initiation (anti-fraud + anchor deposit instruction).

use std::sync::Arc;

use apicash_antifraude::OnRampDecision;
use apicash_auth::JwtClaims;
use apicash_shared::Money;
use axum::extract::State;
use axum::Extension;
use axum::Json;
use serde::Deserialize;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PixPaymentRequest {
    pub user_id: Uuid,
    /// Decimal string (e.g. `"50.00"`).
    pub amount: String,
    pub cpf: String,
    pub social_links: Vec<String>,
    #[serde(default)]
    pub memo: Option<String>,
}

impl PixPaymentRequest {
    fn validate(&self) -> Result<(), &'static str> {
        let digits: String = self.cpf.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() != 11 {
            return Err("cpf must have 11 digits");
        }
        if self.amount.trim().is_empty() {
            return Err("amount is required");
        }
        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct PixPaymentResponse {
    pub stellar_tx_hash: String,
    pub status: String,
    /// EMV BR Code payload for PIX copy/paste.
    pub pix_br_code: String,
}

#[instrument(skip(state, req), fields(user_id = %req.user_id))]
pub async fn create_pix_payment(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<JwtClaims>>,
    Json(req): Json<PixPaymentRequest>,
) -> Result<Json<PixPaymentResponse>, ApiError> {
    req.validate().map_err(ApiError::bad_request)?;

    // Security rule: for sensitive actions, prefer the authenticated identity (JWT `sub`) over a
    // caller-provided `user_id`. In dev mode (`auth_disabled`), keep behavior permissive.
    if !state.auth.config().auth_disabled {
        let Some(Extension(c)) = claims else {
            return Err(ApiError::unauthorized("missing JWT"));
        };
        if c.sub != req.user_id {
            warn!(
                token_user_id = %c.sub,
                requested_user_id = %req.user_id,
                "PIX denied: user_id mismatch"
            );
            return Err(ApiError::unauthorized(
                "user_id must match the authenticated user",
            ));
        }
    }

    let amount = Money::from_str_strict(req.amount.trim()).map_err(|_| {
        error!("invalid amount for PIX");
        ApiError::bad_request("invalid amount decimal")
    })?;

    let cpf: String = req.cpf.chars().filter(|c| c.is_ascii_digit()).collect();

    let score = state
        .antifraude
        .calculate_score(req.user_id, &cpf, &req.social_links)
        .await
        .map_err(|e| {
            error!(error = %e, "antifraude failed for PIX");
            ApiError::from(e)
        })?;

    if score.decision == OnRampDecision::Block {
        info!(user_id = %req.user_id, "PIX flow blocked by risk policy");
        return Err(ApiError::forbidden(
            "on-ramp blocked by anti-fraud policy for this user",
        ));
    }

    let memo = req
        .memo
        .clone()
        .unwrap_or_else(|| format!("pix:user:{}", req.user_id));

    let on_ramp = state.anchor.deposit_pix(amount, memo).await.map_err(|e| {
        error!(error = %e, "anchor PIX deposit failed");
        ApiError::from(e)
    })?;

    let pix_br_code = on_ramp.pix_br_code.filter(|s| !s.trim().is_empty());
    let Some(pix_br_code) = pix_br_code else {
        return Err(ApiError::bad_gateway(
            "anchor on-ramp did not return pix_br_code",
        ));
    };

    info!(tx = %on_ramp.stellar_tx_hash, "PIX deposit initiated");

    Ok(Json(PixPaymentResponse {
        stellar_tx_hash: on_ramp.stellar_tx_hash,
        status: on_ramp.status,
        pix_br_code,
    }))
}
