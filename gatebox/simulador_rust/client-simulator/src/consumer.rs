// Consumer RabbitMQ - consome client-simulator-queue e chama API Gatebox
// Faz exatamente o que gateboxgo/simulators/client-simulator/consumer.go faz
use futures::StreamExt;
use hmac::{Hmac, Mac};
use lapin::options::*;
use lapin::types::FieldTable;
use sha2::Sha256;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tracing::{error, info, warn};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PixTransaction {
    pub id: String,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub amount: f64,
    #[serde(default)]
    pub from_user_id: i32,
    #[serde(default)]
    pub to_user_id: i32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub target_id: String,
    pub pix_key: Option<PixKeyInfo>,
    pub qr_code: Option<QRCodeInfo>,
    pub recipient_key: Option<PixKeyInfo>,
    pub recipient_qr_code: Option<QRCodeInfo>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PixKeyInfo {
    #[serde(rename = "type")]
    pub key_type: String,
    pub value: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct QRCodeInfo {
    pub qr_code_data: String,
    #[serde(default)]
    pub qr_code_id: String,
    #[serde(default, rename = "static")]
    pub is_static: bool,
}

#[derive(Default)]
pub struct ConsumerStats {
    pub total_processed: AtomicI64,
    pub total_success: AtomicI64,
    pub total_failed: AtomicI64,
}

impl ConsumerStats {
    pub fn to_map(&self) -> serde_json::Value {
        serde_json::json!({
            "running": true,
            "total_processed": self.total_processed.load(Ordering::SeqCst),
            "total_success": self.total_success.load(Ordering::SeqCst),
            "total_failed": self.total_failed.load(Ordering::SeqCst),
            "worker_count": 1,
            "consumers": 1,
            "workers_per_pool": 1,
        })
    }
}

fn hmac_sign(payload: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub async fn call_pix_in_key(api_url: &str, pix_tx: &PixTransaction, client: &reqwest::Client) -> Result<(), String> {
    let pix_key = pix_tx
        .pix_key
        .as_ref()
        .map(|k| k.value.as_str())
        .unwrap_or("test@simulator.com");
    let end_to_end = format!("E{:014}{}", chrono::Utc::now().timestamp(), pix_tx.id);
    let payload = serde_json::json!({
        "endToEndId": end_to_end,
        "amount": pix_tx.amount,
        "pixKey": pix_key,
        "payerName": format!("Pagador Simulado {}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) % 10000),
        "payerDocument": "12345678900",
        "description": format!("PIX recebido - {}", pix_tx.description),
        "idempotencyKey": end_to_end,
        "transactionDate": chrono::Utc::now().to_rfc3339(),
    });
    let json_str = payload.to_string();
    let signature = hmac_sign(&json_str, "webhook_secret_key_simulator");
    let url = format!("{}/api/v1/pix/webhook/in", api_url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-Signature", signature)
        .header("X-Idempotency-Key", &end_to_end)
        .body(json_str)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("API retornou {}", resp.status()))
    }
}

pub async fn call_pix_in_qrcode(api_url: &str, pix_tx: &PixTransaction, client: &reqwest::Client) -> Result<(), String> {
    let pix_key = pix_tx
        .pix_key
        .as_ref()
        .map(|k| k.value.as_str())
        .unwrap_or("test@simulator.com");
    let qr_payload = serde_json::json!({
        "amount": pix_tx.amount,
        "payerName": "Cliente Simulado",
        "payerDocument": "12345678900",
        "description": format!("QR Code Simulado - {}", pix_tx.description),
        "expirationSeconds": 1800,
        "reference": pix_tx.id,
        "pixKey": pix_key,
    });
    let qr_url = format!("{}/api/v1/pix/qrcode?userId=3", api_url.trim_end_matches('/'));
    let qr_resp = client
        .post(&qr_url)
        .json(&qr_payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !qr_resp.status().is_success() {
        return Err(format!("QRCode API retornou {}", qr_resp.status()));
    }
    let qr_body: serde_json::Value = qr_resp.json().await.map_err(|e| e.to_string())?;
    let end_to_end = qr_body
        .get("txId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("E{:014}{}", chrono::Utc::now().timestamp(), pix_tx.id));
    let webhook_payload = serde_json::json!({
        "endToEndId": end_to_end,
        "amount": pix_tx.amount,
        "pixKey": pix_key,
        "payerName": "Pagador QR Code",
        "payerDocument": "12345678900",
        "description": format!("PIX via QR Code - {}", pix_tx.description),
        "idempotencyKey": end_to_end,
        "transactionDate": chrono::Utc::now().to_rfc3339(),
        "isQRCodePayment": true,
    });
    let json_str = webhook_payload.to_string();
    let signature = hmac_sign(&json_str, "webhook_secret_key_simulator");
    let webhook_url = format!("{}/api/v1/pix/webhook/in", api_url.trim_end_matches('/'));
    let webhook_resp = client
        .post(&webhook_url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-Signature", signature)
        .header("X-Idempotency-Key", &end_to_end)
        .body(json_str)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if webhook_resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Webhook retornou {}", webhook_resp.status()))
    }
}

pub async fn call_pix_out(api_url: &str, pix_tx: &PixTransaction, client: &reqwest::Client) -> Result<(), String> {
    let (pix_key, type_key) = if let Some(ref rk) = pix_tx.recipient_key {
        (rk.value.clone(), rk.key_type.clone())
    } else if let Some(ref rq) = pix_tx.recipient_qr_code {
        (rq.qr_code_data.clone(), "RANDOM".to_string())
    } else {
        return Err("PIX OUT sem RecipientKey nem RecipientQRCode".to_string());
    };
    let payload = serde_json::json!({
        "key": pix_key,
        "typeKey": type_key,
        "amount": pix_tx.amount,
        "name": "Cliente Simulado",
        "documentNumber": "12345678900",
        "description": format!("Simulação PIX - {}", pix_tx.description),
        "externalId": pix_tx.id,
    });
    let url = format!("{}/api/v1/pix/send?userId=3", api_url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("API retornou {}", resp.status()))
    }
}

pub fn call_gatebox_api(tx_type: &str) -> bool {
    matches!(
        tx_type,
        "pix_in_key" | "pix_in_qrcode" | "pix_out_key" | "pix_out_qrcode" | "pix_out_dict"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_gatebox_api() {
        assert!(call_gatebox_api("pix_in_key"));
        assert!(call_gatebox_api("pix_out_key"));
        assert!(call_gatebox_api("pix_out_qrcode"));
        assert!(call_gatebox_api("pix_out_dict"));
        assert!(!call_gatebox_api("unknown"));
    }

    #[test]
    fn test_pix_transaction_deserialize() {
        let json = r#"{"id":"tx1","type":"pix_out_key","amount":10.5,"recipientKey":{"type":"EMAIL","value":"a@b.com"}}"#;
        let tx: PixTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.id, "tx1");
        assert_eq!(tx.tx_type, "pix_out_key");
        assert!((tx.amount - 10.5).abs() < 0.001);
        assert_eq!(tx.recipient_key.as_ref().unwrap().value, "a@b.com");
        assert_eq!(tx.recipient_key.as_ref().unwrap().key_type, "EMAIL");
    }
}

pub async fn run_consumer(
    rabbitmq_url: &str,
    queue_name: &str,
    api_url: &str,
    stats: Arc<ConsumerStats>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = lapin::Connection::connect(rabbitmq_url, lapin::ConnectionProperties::default())
        .await
        .map_err(|e| anyhow::anyhow!("rabbitmq connect: {}", e))?;
    let channel = conn
        .create_channel()
        .await
        .map_err(|e| anyhow::anyhow!("create channel: {}", e))?;
    channel
        .queue_declare(queue_name, QueueDeclareOptions::default(), FieldTable::default())
        .await
        .map_err(|e| anyhow::anyhow!("queue_declare: {}", e))?;
    channel
        .basic_qos(100, BasicQosOptions::default())
        .await
        .map_err(|e| anyhow::anyhow!("basic_qos: {}", e))?;
    let mut consumer = channel
        .basic_consume(
            queue_name,
            "client-simulator-rust",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("basic_consume: {}", e))?;

    info!(
        "Client Simulator RabbitMQ consumer started: queue={}, api={}",
        queue_name, api_url
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    while let Some(delivery) = consumer.next().await {
        let d = match delivery {
            Ok(d) => d,
            Err(e) => {
                error!("Consumer delivery error: {}", e);
                continue;
            }
        };

        stats.total_processed.fetch_add(1, Ordering::SeqCst);

        let pix_tx: PixTransaction = match serde_json::from_slice(&d.data) {
            Ok(t) => t,
            Err(e) => {
                warn!("Erro ao deserializar mensagem: {} - DESCARTANDO", e);
                let _ = d
                    .nack(BasicNackOptions {
                        multiple: false,
                        requeue: false,
                    })
                    .await;
                stats.total_failed.fetch_add(1, Ordering::SeqCst);
                continue;
            }
        };

        if !call_gatebox_api(&pix_tx.tx_type) {
            let _ = d.ack(BasicAckOptions::default()).await;
            stats.total_success.fetch_add(1, Ordering::SeqCst);
            continue;
        }

        let result = match pix_tx.tx_type.as_str() {
            "pix_in_key" => call_pix_in_key(api_url, &pix_tx, &client).await,
            "pix_in_qrcode" => call_pix_in_qrcode(api_url, &pix_tx, &client).await,
            "pix_out_key" | "pix_out_qrcode" | "pix_out_dict" => call_pix_out(api_url, &pix_tx, &client).await,
            _ => Ok(()),
        };

        match result {
            Ok(()) => {
                let _ = d.ack(BasicAckOptions::default()).await;
                stats.total_success.fetch_add(1, Ordering::SeqCst);
                let p = stats.total_processed.load(Ordering::SeqCst);
                if p % 100 == 0 {
                    info!(
                        "Processadas: {} | Sucesso: {} | Falha: {}",
                        p,
                        stats.total_success.load(Ordering::SeqCst),
                        stats.total_failed.load(Ordering::SeqCst)
                    );
                }
            }
            Err(e) => {
                warn!("Erro ao chamar API: {} - DESCARTANDO", e);
                let _ = d
                    .nack(BasicNackOptions {
                        multiple: false,
                        requeue: false,
                    })
                    .await;
                stats.total_failed.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    Ok(())
}
