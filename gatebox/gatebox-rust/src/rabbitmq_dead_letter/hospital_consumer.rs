// POC: Consume from RabbitMQ hospital queue (from app/modules/core/rabbitmq/hospital_consumer.go)
use anyhow::Result;
use futures::StreamExt;
use lapin::options::*;
use lapin::Channel;
use rust_decimal::Decimal;
use std::sync::Arc;
use tracing::{error, info, warn};

use super::config::{rabbitmq_uri, QUEUE_NAME_HOSPITAL};
use super::types::PaymentMessage;

/// Extracts x-retry-count from AMQP headers if present. POC: iterates keys (avoids extra dep).
fn get_retry_count(headers: &Option<lapin::types::FieldTable>) -> i32 {
    let table = match headers {
        Some(t) => t,
        None => return 0,
    };
    for (k, v) in table.inner().iter() {
        if k.as_str() == "x-retry-count" {
            return v
                .as_long_int()
                .or_else(|| v.as_long_long_int().map(|i| i as i32))
                .unwrap_or(0);
        }
    }
    0
}

/// Process one hospital delivery: parse body as PaymentMessage, log, persist, ack.
pub async fn handle_hospital_delivery(
    delivery: lapin::message::Delivery,
    _channel: &Channel,
    hospital_repo: Option<Arc<dyn crate::hospital_message::HospitalMessageRepository>>,
) -> Result<()> {
    let retry_count = get_retry_count(&delivery.properties.headers());
    let body = delivery.data.as_slice();

    let msg: Result<PaymentMessage, _> = serde_json::from_slice(body);
    match &msg {
        Ok(m) => {
            error!(
                payment_id = m.payment_id,
                amount = m.amount,
                retry_count = retry_count,
                "CRITICAL: Message sent to hospital after max retries"
            );
        }
        Err(e) => {
            error!(
                err = %e,
                body = ?std::str::from_utf8(body).unwrap_or(""),
                "Failed to unmarshal hospital message"
            );
        }
    }

    // Persist for manual review
    if let Ok(m) = &msg {
        if let Some(ref repo) = hospital_repo {
            let payload_json = std::str::from_utf8(body).unwrap_or("{}");
            let amount = Decimal::try_from(m.amount).unwrap_or(Decimal::ZERO);
            if let Err(e) = repo.insert(&m.payment_id.to_string(), amount, retry_count, payload_json).await {
                error!("Failed to persist hospital message: {}", e);
            }
        } else {
            warn!(
                payment_id = m.payment_id,
                amount = m.amount,
                retry_count = retry_count,
                "Hospital message (no repo) for manual review"
            );
        }
    }

    delivery.ack(BasicAckOptions::default()).await?;
    Ok(())
}

/// Run hospital consumer: connect, declare queue, consume and process until cancelled.
pub async fn run_hospital_consumer(
    mut cancel: tokio::sync::oneshot::Receiver<()>,
    hospital_repo: Option<Arc<dyn crate::hospital_message::HospitalMessageRepository>>,
) -> Result<()> {
    let uri = rabbitmq_uri();
    let conn = lapin::Connection::connect(&uri, lapin::ConnectionProperties::default()).await?;
    info!("RabbitMQ connected (hospital consumer)");

    let channel = conn.create_channel().await?;
    channel
        .queue_declare(
            QUEUE_NAME_HOSPITAL,
            QueueDeclareOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await?;
    info!("Queue '{}' declared", QUEUE_NAME_HOSPITAL);

    let mut consumer = channel
        .basic_consume(
            QUEUE_NAME_HOSPITAL,
            "hospital-consumer-rust",
            BasicConsumeOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await?;

    info!("Hospital consumer started, waiting for messages on '{}'", QUEUE_NAME_HOSPITAL);

    loop {
        tokio::select! {
            _ = &mut cancel => {
                info!("Hospital consumer cancelled, stopping");
                break;
            }
            delivery = consumer.next() => {
                let d = match delivery {
                    Some(Ok(d)) => d,
                    Some(Err(e)) => {
                        error!("Consumer error: {}", e);
                        continue;
                    }
                    None => break,
                };
                if let Err(e) = handle_hospital_delivery(d, &channel, hospital_repo.clone()).await {
                    error!("handle_hospital_delivery: {}", e);
                }
            }
        }
    }

    Ok(())
}
