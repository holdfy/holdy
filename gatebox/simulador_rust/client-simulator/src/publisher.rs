// Publisher RabbitMQ - publica PixTransaction na fila client-simulator-queue
// Faz exatamente o que gateboxgo/simulators/client-simulator/rabbitmq-client.go faz
use lapin::options::*;
use lapin::types::FieldTable;
use std::sync::atomic::Ordering;

use crate::consumer::{PixKeyInfo, PixTransaction, QRCodeInfo};

#[derive(Clone)]
pub struct RabbitMQTargetConfig {
    pub rabbitmq_url: String,
    pub queue_name: String,
    pub exchange_name: String,
    pub target_transactions: usize,
    pub min_amount: f64,
    pub max_amount: f64,
    pub concurrency: usize,
}

fn generate_pix_transaction(
    target_id: &str,
    i: usize,
    amount: f64,
    tx_type: &str,
) -> PixTransaction {
    let id = format!(
        "pix_{}_{}_{}",
        tx_type,
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        i
    );
    let description = format!("Transação PIX {} - {}", tx_type, amount);

    let (recipient_key, recipient_qr_code, pix_key, qr_code) = match tx_type {
        "pix_out_key" => (
            Some(PixKeyInfo {
                key_type: "EMAIL".to_string(),
                value: "simulator@test.com".to_string(),
            }),
            None,
            None,
            None,
        ),
        "pix_out_qrcode" => (
            None,
            Some(QRCodeInfo {
                qr_code_data: format!("00020126580014br.gov.bcb.pix0136{}", uuid::Uuid::new_v4()),
                qr_code_id: format!("qr_{}", i),
                is_static: false,
            }),
            None,
            None,
        ),
        "pix_in_key" => (
            None,
            None,
            Some(PixKeyInfo {
                key_type: "EMAIL".to_string(),
                value: "test@simulator.com".to_string(),
            }),
            None,
        ),
        "pix_in_qrcode" => (
            None,
            None,
            None,
            Some(QRCodeInfo {
                qr_code_data: format!("00020126580014br.gov.bcb.pix0136{}", uuid::Uuid::new_v4()),
                qr_code_id: format!("qr_in_{}", i),
                is_static: false,
            }),
        ),
        _ => (
            Some(PixKeyInfo {
                key_type: "EMAIL".to_string(),
                value: "simulator@test.com".to_string(),
            }),
            None,
            None,
            None,
        ),
    };

    PixTransaction {
        id: id.clone(),
        tx_type: tx_type.to_string(),
        amount,
        from_user_id: 1,
        to_user_id: 2,
        description,
        target_id: target_id.to_string(),
        pix_key,
        qr_code,
        recipient_key,
        recipient_qr_code,
    }
}

pub async fn run_target_via_rabbitmq(
    config: RabbitMQTargetConfig,
    target_id: String,
    progress: std::sync::Arc<std::sync::atomic::AtomicI64>,
    targets: std::sync::Arc<
        tokio::sync::RwLock<std::collections::HashMap<String, crate::PixTargetResponse>>,
    >,
) {
    let conn = match lapin::Connection::connect(
        &config.rabbitmq_url,
        lapin::ConnectionProperties::default(),
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Erro ao conectar RabbitMQ: {}", e);
            let mut t = targets.write().await;
            if let Some(target) = t.get_mut(&target_id) {
                target.status = "error".to_string();
                target.message = format!("Erro RabbitMQ: {}", e);
            }
            return;
        }
    };

    let channel = match conn.create_channel().await {
        Ok(ch) => ch,
        Err(e) => {
            tracing::error!("Erro ao criar canal: {}", e);
            return;
        }
    };

    channel
        .exchange_declare(
            &config.exchange_name,
            lapin::ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .ok();

    channel
        .queue_declare(
            &config.queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .ok();

    channel
        .queue_bind(
            &config.queue_name,
            &config.exchange_name,
            "pix.transaction",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .ok();

    let per_worker = config.target_transactions / config.concurrency;
    let remainder = config.target_transactions % config.concurrency;

    let mut handles = Vec::new();
    for w in 0..config.concurrency {
        let n = per_worker + if w < remainder { 1 } else { 0 };
        if n == 0 {
            continue;
        }
        let channel = conn.create_channel().await.ok().expect("channel");
        let exchange = config.exchange_name.clone();
        let target_id = target_id.clone();
        let min_a = config.min_amount;
        let max_a = config.max_amount;
        let progress = std::sync::Arc::clone(&progress);

        handles.push(tokio::spawn(async move {
            for i in 0..n {
                let amount = if max_a > min_a {
                    min_a + rand::random::<f64>() * (max_a - min_a)
                } else {
                    min_a
                };
                let tx = generate_pix_transaction(&target_id, i, amount, "pix_out_key");
                let body = serde_json::to_vec(&tx).unwrap_or_default();
                if channel
                    .basic_publish(
                        &exchange,
                        "pix.transaction",
                        BasicPublishOptions::default(),
                        &body,
                        lapin::BasicProperties::default()
                            .with_delivery_mode(2)
                            .with_content_type("application/json".into()),
                    )
                    .await
                    .is_ok()
                {
                    progress.fetch_add(1, Ordering::SeqCst);
                }
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    let mut t = targets.write().await;
    if let Some(target) = t.get_mut(&target_id) {
        target.status = "completed".to_string();
        target.message = "Target via RabbitMQ concluído".to_string();
    }

    tracing::info!(
        "RabbitMQ target {} concluído: {} mensagens publicadas",
        target_id,
        progress.load(Ordering::SeqCst)
    );
}
