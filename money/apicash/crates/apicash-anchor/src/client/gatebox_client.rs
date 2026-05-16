//! HTTP client minimal para PIX dinâmico no Gatebox (`POST /api/v1/pix/qrcode`).
//! Com `APICASH_FIAT_RAIL=simulated`, o PIX EMV vem **só** daqui (âncora não inventa payload local).

use std::time::Duration;

use apicash_shared::Money;
use reqwest::Client;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::errors::AnchorError;

#[derive(Debug, Serialize)]
struct QrcodeRequestBody<'a> {
    amount: f64,
    payer_name: &'a str,
    payer_document: &'a str,
    description: &'a str,
    expiration_seconds: i32,
    reference: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pix_key: Option<&'a str>,
}

/// Resposta camelCase igual a [`GenerateQrCodeResponse`] no Gatebox.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GateboxQrCodeParsed {
    pub status_code: i32,
    pub qr_code: String,
    #[serde(default)]
    pub tx_id: String,
    #[serde(default)]
    pub transaction_id: String,
    #[serde(default)]
    pub gateway: String,
}

fn gatebox_timeout() -> Duration {
    let ms: u64 = std::env::var("GATEBOX_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);
    Duration::from_millis(ms.max(500).min(120_000))
}

fn expire_seconds_or_default() -> i32 {
    std::env::var("GATEBOX_QR_EXPIRATION_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1800)
        .clamp(60, 86_400)
}

/// Gera código PIX BR (dinâmico / copia-e-cola EMV + QR) através do Gatebox já em execução.
pub async fn fetch_dynamic_pix_qrcode(
    http: &Client,
    amount: Money,
    memo: &str,
) -> Result<GateboxQrCodeParsed, AnchorError> {
    let base = std::env::var("GATEBOX_BASE_URL")
        .map_err(|_| AnchorError::Config("GATEBOX_BASE_URL missing".into()))?
        .trim_end_matches('/')
        .to_string();
    let url = format!("{base}/api/v1/pix/qrcode");

    let amt = amount
        .decimal()
        .to_f64()
        .ok_or_else(|| AnchorError::Validation("amount not representable as f64".into()))?;

    let payer_name = std::env::var("GATEBOX_CUSTOMER_NAME").unwrap_or_else(|_| "HoldFy payer".into());
    let payer_document = std::env::var("GATEBOX_CUSTOMER_DOCUMENT").unwrap_or_default();
    let pix_key_var = std::env::var("GATEBOX_DEFAULT_PIX_KEY").unwrap_or_default();
    let pix_key_trim = pix_key_var.trim();
    let pix_key = (!pix_key_trim.is_empty()).then_some(pix_key_trim);

    let desc = format!("HoldFy {memo}");
    let body = QrcodeRequestBody {
        amount: amt,
        payer_name: payer_name.trim(),
        payer_document: payer_document.trim(),
        description: desc.as_str(),
        expiration_seconds: expire_seconds_or_default(),
        reference: memo,
        pix_key,
    };

    let timeout = gatebox_timeout();
    let api_key_opt = std::env::var("GATEBOX_API_KEY").ok().filter(|s| !s.trim().is_empty());

    let mut req = http.post(&url).json(&body).timeout(timeout);

    if let Some(ref key) = api_key_opt {
        req = req.bearer_auth(key);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| AnchorError::Unavailable(format!("gatebox qrcode unreachable: {e}")))?;

    let status = resp.status();
    let txt = resp
        .text()
        .await
        .map_err(|e| AnchorError::Http(e))?;

    if !status.is_success() {
        return Err(AnchorError::Anchor(format!(
            "gatebox /pix/qrcode HTTP {} — {txt}",
            status.as_u16()
        )));
    }

    let parsed: GateboxQrCodeParsed =
        serde_json::from_str(&txt).map_err(|e| AnchorError::Anchor(format!(
            "gatebox /pix/qrcode invalid JSON ({e}): {txt}"
        )))?;

    if parsed.qr_code.trim().is_empty() {
        return Err(AnchorError::Anchor(
            "gatebox returned empty qr_code (EMV)".into(),
        ));
    }

    if parsed.status_code < 200 || parsed.status_code > 299 {
        return Err(AnchorError::Anchor(format!(
            "gatebox qr_code rejected: status_code={} gateway={}",
            parsed.status_code, parsed.gateway,
        )));
    }

    Ok(parsed)
}
