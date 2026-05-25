//! Circuit breaker para provedores de rastreio.
//!
//! Estados: Closed (normal) → Open (falhou demais) → HalfOpen (testando recuperação).

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug)]
enum BreakerState {
    Closed { failures: u32 },
    Open { opened_at: Instant },
    HalfOpen,
}

impl Default for BreakerState {
    fn default() -> Self {
        BreakerState::Closed { failures: 0 }
    }
}

/// Circuit breaker thread-safe para um único provedor.
pub struct CircuitBreaker {
    name: String,
    failure_threshold: u32,
    reset_timeout: Duration,
    state: RwLock<BreakerState>,
}

impl CircuitBreaker {
    pub fn new(name: impl Into<String>, failure_threshold: u32, reset_timeout_secs: u64) -> Arc<Self> {
        Arc::new(Self {
            name: name.into(),
            failure_threshold,
            reset_timeout: Duration::from_secs(reset_timeout_secs),
            state: RwLock::new(BreakerState::default()),
        })
    }

    /// Retorna `true` se o provedor está disponível para receber chamadas.
    /// Transiciona `Open → HalfOpen` quando o timeout expirar.
    pub async fn is_available(&self) -> bool {
        {
            let guard = self.state.read().await;
            match &*guard {
                BreakerState::Closed { .. } => return true,
                BreakerState::HalfOpen => return true,
                BreakerState::Open { opened_at } => {
                    if opened_at.elapsed() < self.reset_timeout {
                        return false;
                    }
                    // Timeout expirou — precisamos de write lock para transicionar.
                }
            }
        }

        // Só chega aqui se estava Open e o timeout expirou.
        let mut guard = self.state.write().await;
        if let BreakerState::Open { opened_at } = &*guard {
            if opened_at.elapsed() >= self.reset_timeout {
                info!(provider = %self.name, "circuit_breaker: Open → HalfOpen (timeout expirou)");
                *guard = BreakerState::HalfOpen;
                return true;
            }
        }
        false
    }

    /// Registra sucesso: reseta para `Closed { failures: 0 }`.
    pub async fn record_success(&self) {
        let mut guard = self.state.write().await;
        match &*guard {
            BreakerState::HalfOpen | BreakerState::Open { .. } => {
                info!(provider = %self.name, "circuit_breaker: → Closed (sucesso)");
            }
            BreakerState::Closed { failures } if *failures > 0 => {
                info!(provider = %self.name, "circuit_breaker: falhas zeradas");
            }
            _ => {}
        }
        *guard = BreakerState::Closed { failures: 0 };
    }

    /// Registra falha: incrementa contador e abre o circuito ao atingir o limiar.
    pub async fn record_failure(&self) {
        let mut guard = self.state.write().await;
        match &*guard {
            BreakerState::Closed { failures } => {
                let new_failures = failures + 1;
                if new_failures >= self.failure_threshold {
                    warn!(
                        provider = %self.name,
                        failures = new_failures,
                        "circuit_breaker: Closed → Open (limiar atingido)"
                    );
                    *guard = BreakerState::Open { opened_at: Instant::now() };
                } else {
                    *guard = BreakerState::Closed { failures: new_failures };
                }
            }
            BreakerState::HalfOpen => {
                warn!(provider = %self.name, "circuit_breaker: HalfOpen → Open (falhou na prova)");
                *guard = BreakerState::Open { opened_at: Instant::now() };
            }
            BreakerState::Open { .. } => {
                // Já aberto, nada a fazer.
            }
        }
    }
}
