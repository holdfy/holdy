//! Provedor Correios Business API (autenticação por cartão postagem).
//!
//! Variáveis de ambiente:
//! - `CORREIOS_USER`        — usuário (ex.: CPF/CNPJ)
//! - `CORREIOS_ACCESS_CODE` — código de acesso do cartão postagem

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::error::LogisticsError;
use crate::tracking::circuit_breaker::CircuitBreaker;
use crate::tracking::map_status_from_description;
use crate::types::{TrackingEvent, TrackingInfo, TrackingStatus};

use super::TrackingProvider;

const AUTH_URL: &str = "https://api.correios.com.br/token/v1/autentica/cartaopostagem";
const TRACKING_BASE: &str = "https://api.correios.com.br/srorastro/v1/objetos";

/// Token em cache com timestamp de obtenção.
struct CachedToken {
    token: String,
    obtained_at: Instant,
}

pub struct CorreiosProvider {
    user: String,
    access_code: String,
    client: Client,
    cached_token: RwLock<Option<CachedToken>>,
    pub breaker: Arc<CircuitBreaker>,
}

impl CorreiosProvider {
    /// Constrói o provedor se as variáveis de ambiente estiverem presentes.
    pub fn from_env() -> Option<Self> {
        let user = std::env::var("CORREIOS_USER").ok()?;
        let access_code = std::env::var("CORREIOS_ACCESS_CODE").ok()?;
        if user.is_empty() || access_code.is_empty() {
            return None;
        }
        Some(Self {
            user,
            access_code,
            client: Client::new(),
            cached_token: RwLock::new(None),
            breaker: CircuitBreaker::new("correios", 3, 60),
        })
    }

    /// Obtém token (usa cache se ainda válido — ~50 min margem).
    async fn get_token(&self) -> Result<String, LogisticsError> {
        // Verificar cache — tokens Correios expiram em ~1h; usamos 50 min.
        {
            let guard = self.cached_token.read().await;
            if let Some(ct) = &*guard {
                if ct.obtained_at.elapsed() < Duration::from_secs(50 * 60) {
                    return Ok(ct.token.clone());
                }
            }
        }

        self.fetch_token().await
    }

    async fn fetch_token(&self) -> Result<String, LogisticsError> {
        debug!(user = %self.user, "correios: autenticando");
        let resp = self
            .client
            .post(AUTH_URL)
            .basic_auth(&self.user, Some(&self.access_code))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| LogisticsError::RequestFailed(format!("correios auth: {e}")))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(LogisticsError::ApiError(
                "correios: credenciais inválidas (401)".into(),
            ));
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(LogisticsError::ApiError(format!(
                "correios auth HTTP {status}: {body}"
            )));
        }

        let json: Value = resp
            .json()
            .await
            .map_err(|e| LogisticsError::RequestFailed(format!("correios auth parse: {e}")))?;

        let token = json
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                LogisticsError::ApiError("correios auth: campo 'token' ausente".into())
            })?
            .to_string();

        let mut guard = self.cached_token.write().await;
        *guard = Some(CachedToken {
            token: token.clone(),
            obtained_at: Instant::now(),
        });
        Ok(token)
    }

    /// Limpa o token em cache e reauthentifica.
    async fn refresh_token(&self) -> Result<String, LogisticsError> {
        {
            let mut guard = self.cached_token.write().await;
            *guard = None;
        }
        self.fetch_token().await
    }

    async fn fetch_tracking(&self, code: &str, token: &str) -> Result<Value, LogisticsError> {
        let url = format!("{TRACKING_BASE}/{code}?resultado=T");
        let resp = self
            .client
            .get(&url)
            .bearer_auth(token)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| LogisticsError::RequestFailed(format!("correios tracking: {e}")))?;

        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(LogisticsError::TrackingNotFound(code.to_string()));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(LogisticsError::ApiError(format!(
                "correios tracking HTTP {status}: {body}"
            )));
        }

        resp.json::<Value>()
            .await
            .map_err(|e| LogisticsError::RequestFailed(format!("correios tracking parse: {e}")))
    }
}

#[async_trait]
impl TrackingProvider for CorreiosProvider {
    fn name(&self) -> &'static str {
        "correios"
    }

    async fn track(&self, code: &str) -> Result<TrackingInfo, LogisticsError> {
        let token = self.get_token().await?;

        let json = match self.fetch_tracking(code, &token).await {
            Err(LogisticsError::ApiError(ref msg)) if msg.contains("401") => {
                warn!("correios: 401 no rastreio, renovando token");
                let new_token = self.refresh_token().await?;
                self.fetch_tracking(code, &new_token).await?
            }
            other => other?,
        };

        parse_correios_response(code, &json)
    }
}

fn parse_correios_response(code: &str, json: &Value) -> Result<TrackingInfo, LogisticsError> {
    let objetos = json
        .get("objetos")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LogisticsError::ApiError("correios: campo 'objetos' ausente".into()))?;

    let obj = objetos
        .first()
        .ok_or_else(|| LogisticsError::TrackingNotFound(code.to_string()))?;

    let eventos = obj
        .get("eventos")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut events: Vec<TrackingEvent> = eventos
        .iter()
        .filter_map(|ev| parse_correios_event(ev))
        .collect();

    // Mais recente primeiro (Correios devolve cronológico — inverter).
    events.reverse();

    let current_status = events
        .first()
        .map(|e| e.status.clone())
        .unwrap_or(TrackingStatus::Unknown);

    let carrier = obj
        .get("produto")
        .and_then(|v| v.as_str())
        .unwrap_or("Correios")
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

fn parse_correios_event(ev: &Value) -> Option<TrackingEvent> {
    let descricao = ev.get("descricao")?.as_str()?.to_string();
    let dt_str = ev.get("dtHrCriado")?.as_str()?;
    let occurred_at = chrono::DateTime::parse_from_rfc3339(dt_str)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);

    let cidade = ev
        .get("unidade")
        .and_then(|u| u.get("endereco"))
        .and_then(|e| e.get("cidade"))
        .and_then(|v| v.as_str());
    let uf = ev
        .get("unidade")
        .and_then(|u| u.get("endereco"))
        .and_then(|e| e.get("uf"))
        .and_then(|v| v.as_str());

    let location = match (cidade, uf) {
        (Some(c), Some(u)) => Some(format!("{c}/{u}")),
        (Some(c), None) => Some(c.to_string()),
        _ => None,
    };

    let status = map_status_from_description(&descricao);

    Some(TrackingEvent {
        status,
        description: descricao,
        location,
        occurred_at,
    })
}
