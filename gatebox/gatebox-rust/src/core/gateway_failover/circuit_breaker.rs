// From app/modules/core/gateway_failover/circuit_breaker.go
// In-memory circuit breaker; optional Redis/observability can be wired later.

use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Gateway working normally.
    Closed,
    /// Gateway blocked (failed).
    Open,
    /// Testing if gateway recovered.
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "CLOSED"),
            CircuitState::Open => write!(f, "OPEN"),
            CircuitState::HalfOpen => write!(f, "HALF_OPEN"),
        }
    }
}

/// In-memory circuit breaker for a gateway.
pub struct CircuitBreaker {
    gateway_name: String,
    state: RwLock<State>,
    error_threshold: u32,
    success_threshold: u32,
    open_duration: Duration,
    half_open_max_attempts: u32,
}

#[derive(Debug)]
struct State {
    circuit_state: CircuitState,
    consecutive_errors: u32,
    consecutive_successes: u32,
    last_error_at: Option<Instant>,
    last_success_at: Option<Instant>,
    circuit_opened_at: Option<Instant>,
    last_state_change: Instant,
    half_open_attempts: u32,
}

impl CircuitBreaker {
    pub fn new(
        gateway_name: String,
        error_threshold: u32,
        success_threshold: u32,
        open_duration: Duration,
    ) -> Self {
        Self {
            gateway_name,
            state: RwLock::new(State {
                circuit_state: CircuitState::Closed,
                consecutive_errors: 0,
                consecutive_successes: 0,
                last_error_at: None,
                last_success_at: None,
                circuit_opened_at: None,
                last_state_change: Instant::now(),
                half_open_attempts: 0,
            }),
            error_threshold,
            success_threshold,
            open_duration,
            half_open_max_attempts: 1,
        }
    }

    pub fn gateway_name(&self) -> &str {
        &self.gateway_name
    }

    /// Returns current state; may transition Open -> HalfOpen if open_duration elapsed.
    pub fn get_state(&self) -> CircuitState {
        let mut guard = self.state.write().unwrap();
        if guard.circuit_state == CircuitState::Open {
            if let Some(opened_at) = guard.circuit_opened_at {
                if opened_at.elapsed() >= self.open_duration {
                    self.transition_to_half_open(&mut guard);
                }
            }
        }
        guard.circuit_state
    }

    /// Returns true if a call to the gateway is allowed.
    pub fn can_attempt(&self) -> bool {
        let state = self.get_state();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => {
                let mut guard = self.state.write().unwrap();
                if guard.half_open_attempts < self.half_open_max_attempts {
                    guard.half_open_attempts += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn record_success(&self) {
        let mut guard = self.state.write().unwrap();
        guard.last_success_at = Some(Instant::now());
        guard.consecutive_errors = 0;
        match guard.circuit_state {
            CircuitState::Closed => {}
            CircuitState::HalfOpen => {
                guard.consecutive_successes += 1;
                if guard.consecutive_successes >= self.success_threshold {
                    self.transition_to_closed(&mut guard);
                }
            }
            CircuitState::Open => {}
        }
    }

    pub fn record_error(&self) {
        let mut guard = self.state.write().unwrap();
        guard.last_error_at = Some(Instant::now());
        guard.consecutive_successes = 0;
        guard.consecutive_errors += 1;
        match guard.circuit_state {
            CircuitState::Closed => {
                if guard.consecutive_errors >= self.error_threshold {
                    self.transition_to_open(&mut guard);
                }
            }
            CircuitState::HalfOpen => {
                self.transition_to_open(&mut guard);
            }
            CircuitState::Open => {}
        }
    }

    pub fn get_consecutive_errors(&self) -> u32 {
        self.state.read().unwrap().consecutive_errors
    }

    fn transition_to_open(&self, guard: &mut State) {
        guard.circuit_state = CircuitState::Open;
        guard.circuit_opened_at = Some(Instant::now());
        guard.last_state_change = Instant::now();
        guard.half_open_attempts = 0;
    }

    fn transition_to_half_open(&self, guard: &mut State) {
        guard.circuit_state = CircuitState::HalfOpen;
        guard.last_state_change = Instant::now();
        guard.consecutive_successes = 0;
        guard.half_open_attempts = 0;
    }

    fn transition_to_closed(&self, guard: &mut State) {
        guard.circuit_state = CircuitState::Closed;
        guard.last_state_change = Instant::now();
        guard.consecutive_errors = 0;
        guard.consecutive_successes = 0;
        guard.half_open_attempts = 0;
        guard.circuit_opened_at = None;
    }

    /// Stats for metrics/debugging.
    pub fn get_stats(&self) -> std::collections::HashMap<String, serde_json::Value> {
        let guard = self.state.read().unwrap();
        let time_in_state = guard.last_state_change.elapsed().as_secs_f64();
        let mut m = std::collections::HashMap::new();
        m.insert(
            "gateway_name".to_string(),
            serde_json::json!(self.gateway_name),
        );
        m.insert(
            "state".to_string(),
            serde_json::json!(guard.circuit_state.to_string()),
        );
        m.insert(
            "consecutive_errors".to_string(),
            serde_json::json!(guard.consecutive_errors),
        );
        m.insert(
            "consecutive_successes".to_string(),
            serde_json::json!(guard.consecutive_successes),
        );
        m.insert(
            "time_in_current_state".to_string(),
            serde_json::json!(format!("{:.2}s", time_in_state)),
        );
        m
    }

    pub fn reset(&self) {
        let mut guard = self.state.write().unwrap();
        guard.circuit_state = CircuitState::Closed;
        guard.consecutive_errors = 0;
        guard.consecutive_successes = 0;
        guard.half_open_attempts = 0;
        guard.last_state_change = Instant::now();
        guard.circuit_opened_at = None;
    }

    pub fn force_open(&self) {
        let mut guard = self.state.write().unwrap();
        if guard.consecutive_errors < self.error_threshold {
            guard.consecutive_errors = self.error_threshold;
        }
        self.transition_to_open(&mut guard);
    }

    pub fn force_close(&self) {
        let mut guard = self.state.write().unwrap();
        self.transition_to_closed(&mut guard);
    }
}
