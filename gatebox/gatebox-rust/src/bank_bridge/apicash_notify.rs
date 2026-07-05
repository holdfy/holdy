//! Notifica apicash-core (`/internal/webhook/pix`) quando o Gatebox confirma pagamento
//! feito pelo app `banco` — equivalente ao já existente para o WhatsApp em `whatsapp_notify.rs`.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{info, warn};

type HmacSha256 = Hmac<Sha256>;

/// POST assíncrono para `/internal/webhook/pix` (não bloqueia o handler HTTP).
/// `gateway_in_tx_id` deve ser o mesmo valor gravado na ordem no momento da geração do QR
/// (ver `synthetic_pix_qrcode_response::transaction_id`).
pub fn spawn_apicash_payment_notify(gateway_in_tx_id: String) {
    tokio::spawn(async move {
        if let Err(e) = notify_apicash(&gateway_in_tx_id).await {
            warn!(error = %e, "bank_bridge: apicash-core webhook notify failed");
        }
    });
}

async fn notify_apicash(gateway_in_tx_id: &str) -> Result<(), String> {
    let base = std::env::var("APICASH_CORE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
    let url = format!("{}/internal/webhook/pix", base.trim_end_matches('/'));

    let body = serde_json::json!({
        "type": "pix_in",
        "transaction_id": gateway_in_tx_id,
        "status": "completed",
    });
    let raw = serde_json::to_vec(&body).map_err(|e| e.to_string())?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;

    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(raw.clone());

    if let Ok(secret) = std::env::var("GATEBOX_WEBHOOK_SECRET") {
        if !secret.trim().is_empty() {
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|e| e.to_string())?;
            mac.update(&raw);
            let sig = hex::encode(mac.finalize().into_bytes());
            req = req.header("X-Webhook-Signature", sig);
        }
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    let status = resp.status();
    if status.is_success() {
        info!(gateway_in_tx_id, %status, "bank_bridge: apicash-core webhook notify OK");
        Ok(())
    } else {
        Err(format!("apicash-core webhook HTTP {status}"))
    }
}
