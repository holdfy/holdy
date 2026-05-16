//! Notifica apicash-whatsapp quando o Gatebox confirma pagamento (integração HoldFy).

use tracing::{info, warn};

/// `charge_id` no formato `order_{uuid}` ou referência completa `GATEBOXRUST:QR|…`.
pub fn payment_reference_for_notify(charge_id: &str) -> Option<String> {
    let cid = charge_id.trim();
    if cid.is_empty() {
        return None;
    }
    if cid.starts_with("GATEBOXRUST:QR|") {
        return Some(cid.to_string());
    }
    if cid.starts_with("order_") {
        return Some(format!("GATEBOXRUST:QR|{cid}|0"));
    }
    None
}

/// POST assíncrono para `/internal/bank-payment-notify-by-ref` (não bloqueia o handler HTTP).
pub fn spawn_whatsapp_payment_notify(payment_reference: String) {
    tokio::spawn(async move {
        if let Err(e) = notify_whatsapp_by_reference(&payment_reference).await {
            warn!(error = %e, ref_len = payment_reference.len(), "bank_bridge: whatsapp notify failed");
        }
    });
}

async fn notify_whatsapp_by_reference(payment_reference: &str) -> Result<(), String> {
    let wa_base = std::env::var("APICASH_WHATSAPP_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3010".into());
    let api_key = std::env::var("APICASH_API_KEY").map_err(|_| "APICASH_API_KEY not set")?;
    if api_key.trim().is_empty() {
        return Err("APICASH_API_KEY empty".into());
    }

    let url = format!(
        "{}/internal/bank-payment-notify-by-ref",
        wa_base.trim_end_matches('/')
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("X-Api-Key", api_key.trim())
        .json(&serde_json::json!({ "payment_reference": payment_reference }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    if status.is_success() {
        info!(
            status = %status,
            ref_len = payment_reference.len(),
            "bank_bridge: whatsapp payment notify OK"
        );
        Ok(())
    } else {
        Err(format!("whatsapp notify HTTP {status}"))
    }
}
