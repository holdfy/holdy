use std::sync::Arc;

use reqwest::Client;
use serde::Serialize;
use tracing::{info, warn};

#[derive(Clone)]
pub struct WhatsAppNotifier {
    client: Client,
    url: Option<String>,
    api_key: String,
}

impl WhatsAppNotifier {
    pub fn from_env() -> Self {
        let url = std::env::var("LOGISTICA_WHATSAPP_NOTIFY_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let host = std::env::var("MONEY_LAN_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
                let port = std::env::var("APICASH_WA_WEBHOOK_BIND")
                    .ok()
                    .and_then(|b| b.rsplit(':').next().map(str::to_string))
                    .unwrap_or_else(|| "3010".to_string());
                Some(format!("http://{}:{port}/internal/tracking-step-notify", host.trim()))
            });
        let api_key = std::env::var("APICASH_API_KEY").unwrap_or_default();
        if url.is_some() && api_key.is_empty() {
            warn!("LOGISTICA_WHATSAPP_NOTIFY_URL configurado mas APICASH_API_KEY vazio — notify retornará 401");
        }
        Self {
            client: Client::new(),
            url,
            api_key,
        }
    }

    pub fn spawn_notify(
        self: &Arc<Self>,
        seller_phone: String,
        order_id: Option<String>,
        tracking_code: String,
        step_label: String,
        description: String,
    ) {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            this.notify_step(
                &seller_phone,
                order_id.as_deref(),
                &tracking_code,
                &step_label,
                &description,
            )
            .await;
        });
    }

    async fn notify_step(
        &self,
        seller_phone: &str,
        order_id: Option<&str>,
        tracking_code: &str,
        step_label: &str,
        description: &str,
    ) {
        let Some(url) = &self.url else {
            info!(code = %tracking_code, "whatsapp notify: desativado");
            return;
        };
        if self.api_key.is_empty() {
            warn!(code = %tracking_code, "whatsapp notify: APICASH_API_KEY ausente");
            return;
        }
        let seller = normalize_phone(seller_phone);
        if seller.is_empty() {
            info!(code = %tracking_code, "whatsapp notify: telefone do vendedor não configurado");
            return;
        }

        let body = NotifyBody {
            seller_phone: seller,
            order_id: order_id.map(str::to_string),
            tracking_code: tracking_code.to_string(),
            step_label: step_label.to_string(),
            description: description.to_string(),
        };

        match self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                info!(code = %tracking_code, step = %step_label, "whatsapp notify: enviado ao vendedor");
            }
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                warn!(code = %tracking_code, %status, body = %text, "whatsapp notify: falha HTTP");
            }
            Err(e) => {
                warn!(code = %tracking_code, error = %e, "whatsapp notify: erro de rede");
            }
        }
    }
}

#[derive(Serialize)]
struct NotifyBody {
    seller_phone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    order_id: Option<String>,
    tracking_code: String,
    step_label: String,
    description: String,
}

fn normalize_phone(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return String::new();
    }
    if digits.len() == 11 || digits.len() == 10 {
        return format!("55{digits}");
    }
    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_brazil_mobile() {
        assert_eq!(normalize_phone("11 98765-4321"), "5511987654321");
    }
}
