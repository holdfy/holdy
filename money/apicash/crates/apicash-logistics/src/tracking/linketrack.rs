//! Provedor LinkTrack (https://linketrack.com) — tier gratuito.
//!
//! Variáveis de ambiente:
//! - `LINKETRACK_USER`  — e-mail de cadastro
//! - `LINKETRACK_TOKEN` — token da API

use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use reqwest::Client;
use serde_json::Value;
use tracing::debug;

use crate::error::LogisticsError;
use crate::tracking::circuit_breaker::CircuitBreaker;
use crate::tracking::map_status_from_description;
use crate::types::{TrackingEvent, TrackingInfo, TrackingStatus};

use super::TrackingProvider;

const API_BASE: &str = "https://api.linketrack.com/track/json";

/// Codificação mínima para query strings (substitui espaços e caracteres especiais).
fn simple_urlencode(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~' | '@') {
                vec![c]
            } else {
                format!("%{:02X}", c as u32).chars().collect::<Vec<_>>()
            }
        })
        .collect()
}

pub struct LinkTrackProvider {
    user: String,
    token: String,
    client: Client,
    pub breaker: Arc<CircuitBreaker>,
}

impl LinkTrackProvider {
    /// Constrói o provedor se as variáveis de ambiente estiverem presentes.
    pub fn from_env() -> Option<Self> {
        let user = std::env::var("LINKETRACK_USER").ok()?;
        let token = std::env::var("LINKETRACK_TOKEN").ok()?;
        if user.is_empty() || token.is_empty() {
            return None;
        }
        Some(Self {
            user,
            token,
            client: Client::new(),
            breaker: CircuitBreaker::new("linketrack", 3, 60),
        })
    }
}

#[async_trait]
impl TrackingProvider for LinkTrackProvider {
    fn name(&self) -> &'static str {
        "linketrack"
    }

    async fn track(&self, code: &str) -> Result<TrackingInfo, LogisticsError> {
        let url = format!(
            "{}?user={}&token={}&codigo={}",
            API_BASE,
            simple_urlencode(&self.user),
            simple_urlencode(&self.token),
            simple_urlencode(code)
        );

        debug!(code = %code, "linketrack: consultando rastreio");

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| LogisticsError::RequestFailed(format!("linketrack: {e}")))?;

        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(LogisticsError::TrackingNotFound(code.to_string()));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(LogisticsError::ApiError(format!(
                "linketrack HTTP {status}: {body}"
            )));
        }

        let json: Value = resp
            .json()
            .await
            .map_err(|e| LogisticsError::RequestFailed(format!("linketrack parse: {e}")))?;

        parse_linketrack_response(code, &json)
    }
}

fn parse_linketrack_response(code: &str, json: &Value) -> Result<TrackingInfo, LogisticsError> {
    // Verificar erro da API (campo "erro" ou "message").
    if let Some(err_msg) = json.get("erro").and_then(|v| v.as_str()) {
        if !err_msg.is_empty() && err_msg != "false" && err_msg != "0" {
            return Err(LogisticsError::TrackingNotFound(code.to_string()));
        }
    }

    let carrier = json
        .get("nome")
        .and_then(|v| v.as_str())
        .unwrap_or("Desconhecido")
        .to_string();

    let eventos = json
        .get("eventos")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if eventos.is_empty() {
        return Err(LogisticsError::TrackingNotFound(code.to_string()));
    }

    let events: Vec<TrackingEvent> = eventos
        .iter()
        .filter_map(|ev| parse_linketrack_event(ev))
        .collect();

    let current_status = events
        .first()
        .map(|e| e.status.clone())
        .unwrap_or(TrackingStatus::Unknown);

    Ok(TrackingInfo {
        tracking_code: code.to_string(),
        carrier,
        current_status,
        events,
        estimated_delivery: None,
        provider_used: String::new(), // preenchido pelo CascadingTracker
    })
}

fn parse_linketrack_event(ev: &Value) -> Option<TrackingEvent> {
    let data = ev.get("data")?.as_str()?;
    let hora = ev.get("hora")?.as_str().unwrap_or("00:00");
    let status_text = ev
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let dt_str = format!("{data} {hora}");
    let naive = NaiveDateTime::parse_from_str(&dt_str, "%d/%m/%Y %H:%M").ok()?;
    let occurred_at = naive.and_utc();

    let location = ev
        .get("local")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    // sub_status pode conter detalhes adicionais.
    let sub = ev
        .get("subStatus")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let description = if sub.is_empty() {
        status_text.to_string()
    } else {
        format!("{status_text} — {sub}")
    };

    let status = map_status_from_description(status_text);

    Some(TrackingEvent {
        status,
        description,
        location,
        occurred_at,
    })
}
