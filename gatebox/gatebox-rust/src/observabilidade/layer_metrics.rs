// Converted from gateboxgo/utils/observabilidade/layer_metrics.go
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::observabilidade::metrics;

pub struct LayerTiming {
    usecase: String,
    operation: String,
    layer: String,
    start: Instant,
}

impl LayerTiming {
    pub fn new(usecase: &str, operation: &str, layer: &str) -> Self {
        Self {
            usecase: usecase.to_string(),
            operation: operation.to_string(),
            layer: layer.to_string(),
            start: Instant::now(),
        }
    }

    pub fn finish(&self, err: Option<&anyhow::Error>) {
        let duration = self.start.elapsed();
        let success = err.is_none();
        metrics::record_layer_duration(
            &self.layer,
            &self.usecase,
            &self.operation,
            duration.as_secs_f64(),
            success,
        );
        self.finish_without_duration(err);
    }

    pub fn finish_without_duration(&self, err: Option<&anyhow::Error>) {
        let duration = self.start.elapsed();
        let success = err.is_none();
        let threshold = match self.layer.as_str() {
            "handler" => Duration::from_secs(1),
            "service" => Duration::from_millis(500),
            "repository" => Duration::from_millis(200),
            _ => Duration::from_secs(1),
        };
        if duration > threshold {
            metrics::record_slow_operation(
                &self.usecase,
                &self.operation,
                &self.layer,
                success,
            );
        }
    }
}

pub struct TimeoutDetector {
    usecase: String,
    operation: String,
    start: Instant,
    triggered: AtomicBool,
}

impl TimeoutDetector {
    pub fn new(usecase: &str, operation: &str) -> Self {
        Self {
            usecase: usecase.to_string(),
            operation: operation.to_string(),
            start: Instant::now(),
            triggered: AtomicBool::new(false),
        }
    }

    pub fn check_timeout(&self, ctx_timed_out: bool, ctx_canceled: bool) {
        if ctx_timed_out || ctx_canceled {
            if !self.triggered.swap(true, Ordering::Relaxed) {
                metrics::record_timeout(&self.usecase, &self.operation);
            }
        }
    }
}
