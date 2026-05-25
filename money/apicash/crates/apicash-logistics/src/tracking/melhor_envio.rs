//! Adaptador Melhor Envio como `TrackingProvider` (fallback / prioridade 3).

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::client::MelhorEnvioClient;
use crate::error::LogisticsError;
use crate::tracking::circuit_breaker::CircuitBreaker;
use crate::types::{TrackingEvent, TrackingInfo, TrackingStatus};

use super::TrackingProvider;

pub struct MelhorEnvioTracker {
    client: MelhorEnvioClient,
    pub breaker: Arc<CircuitBreaker>,
}

impl MelhorEnvioTracker {
    pub fn new(client: MelhorEnvioClient) -> Self {
        Self {
            client,
            breaker: CircuitBreaker::new("melhor_envio", 3, 60),
        }
    }
}

#[async_trait]
impl TrackingProvider for MelhorEnvioTracker {
    fn name(&self) -> &'static str {
        "melhor_envio"
    }

    async fn track(&self, code: &str) -> Result<TrackingInfo, LogisticsError> {
        let path = format!("/me/shipment/tracking?orders={code}");
        let resp = self.client.get_json(&path).await?;

        let entry = resp
            .get(code)
            .ok_or_else(|| LogisticsError::TrackingNotFound(code.to_string()))?;

        let current_status = entry
            .get("status")
            .and_then(|v| v.as_str())
            .map(parse_me_status)
            .unwrap_or(TrackingStatus::Unknown);

        let events: Vec<TrackingEvent> = entry
            .get("tracking")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(parse_me_event).collect())
            .unwrap_or_default();

        let carrier = entry
            .get("service")
            .and_then(|v| v.get("company"))
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Desconhecido")
            .to_string();

        Ok(TrackingInfo {
            tracking_code: code.to_string(),
            carrier,
            current_status,
            events,
            estimated_delivery: None,
            provider_used: String::new(), // preenchido pelo CascadingTracker
        })
    }
}

fn parse_me_status(s: &str) -> TrackingStatus {
    match s.to_lowercase().as_str() {
        "posted" => TrackingStatus::Posted,
        "in_transit" | "in transit" => TrackingStatus::InTransit,
        "out_for_delivery" | "out for delivery" => TrackingStatus::OutForDelivery,
        "delivered" => TrackingStatus::Delivered,
        "return_in_progress" => TrackingStatus::ReturnInProgress,
        "returned" => TrackingStatus::Returned,
        "exception" | "incident" => TrackingStatus::Exception,
        _ => TrackingStatus::Unknown,
    }
}

fn parse_me_event(item: &Value) -> Option<TrackingEvent> {
    let description = item.get("message")?.as_str()?.to_string();
    let location = item
        .get("location")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let occurred_at_str = item.get("date")?.as_str()?;
    let occurred_at = chrono::DateTime::parse_from_rfc3339(occurred_at_str)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);
    let status = item
        .get("status")
        .and_then(|v| v.as_str())
        .map(parse_me_status)
        .unwrap_or(TrackingStatus::InTransit);

    Some(TrackingEvent {
        status,
        description,
        location,
        occurred_at,
    })
}
