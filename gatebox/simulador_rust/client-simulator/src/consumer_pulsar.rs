// Consumer Pulsar - consome client-simulator-queue e chama API Gatebox
// Equivalente ao consumer RabbitMQ, usando Pulsar como sistema de mensageria principal
use futures::StreamExt;
use pulsar::SubType;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::consumer::{call_gatebox_api, call_pix_in_key, call_pix_in_qrcode, call_pix_out, ConsumerStats, PixTransaction};

fn get_pulsar_topic() -> String {
    let tenant = std::env::var("PULSAR_TENANT").unwrap_or_else(|_| "public".to_string());
    let namespace = std::env::var("PULSAR_NAMESPACE").unwrap_or_else(|_| "default".to_string());
    format!("persistent://{}/{}/client-simulator-queue", tenant, namespace)
}

pub async fn run_pulsar_consumer(
    pulsar_url: &str,
    api_url: &str,
    stats: Arc<ConsumerStats>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let topic = get_pulsar_topic();

    let pulsar = pulsar::Pulsar::builder(pulsar_url, pulsar::TokioExecutor)
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("pulsar connect: {}", e))?;

    let mut consumer: pulsar::Consumer<Vec<u8>, _> = pulsar
        .consumer()
        .with_topic(topic.clone())
        .with_subscription("client-simulator-sub")
        .with_subscription_type(SubType::Shared)
        .with_consumer_name("client-simulator-rust")
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("pulsar consumer: {}", e))?;

    info!(
        "Client Simulator Pulsar consumer started: topic={}, api={}",
        topic, api_url
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    while let Some(msg_result) = consumer.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                error!("Pulsar consumer delivery error: {}", e);
                continue;
            }
        };

        stats.total_processed.fetch_add(1, Ordering::SeqCst);

        let data: Vec<u8> = msg.deserialize();

        let pix_tx: PixTransaction = match serde_json::from_slice(&data) {
            Ok(t) => t,
            Err(e) => {
                warn!("Erro ao deserializar mensagem: {} - DESCARTANDO", e);
                let _ = consumer.ack(&msg).await;
                stats.total_failed.fetch_add(1, Ordering::SeqCst);
                continue;
            }
        };

        if !call_gatebox_api(&pix_tx.tx_type) {
            let _ = consumer.ack(&msg).await;
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
                let _ = consumer.ack(&msg).await;
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
                let _ = consumer.nack(&msg).await;
                stats.total_failed.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    Ok(())
}
