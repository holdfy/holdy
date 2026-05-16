//! Janela de resposta antes de resolução automática (ex.: 7 dias).

use chrono::{DateTime, Duration, Utc};

/// Configuração de prazo para `auto_resolve_timeout`.
#[derive(Debug, Clone)]
pub struct DisputeTimeoutConfig {
    /// Tempo após abertura sem resolução manual para escalar / auto-resolver.
    pub response_window: Duration,
}

impl Default for DisputeTimeoutConfig {
    fn default() -> Self {
        Self {
            response_window: Duration::days(7),
        }
    }
}

impl DisputeTimeoutConfig {
    /// `true` se a disputa aberta está além da janela configurada.
    pub fn is_past_deadline(&self, opened_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
        opened_at + self.response_window < now
    }

    pub fn deadline(&self, opened_at: DateTime<Utc>) -> DateTime<Utc> {
        opened_at + self.response_window
    }
}
