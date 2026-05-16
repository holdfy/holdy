// Converted from gateboxgo/utils/observabilidade/tracker.go
use std::collections::HashMap;
use std::time::Instant;

use crate::observabilidade::layer_metrics::{LayerTiming, TimeoutDetector};
use crate::observabilidade::metrics;

pub struct ObservabilityTracker {
    pub usecase: String,
    pub operation: String,
    pub layer: String,
    pub start: Instant,
    params: HashMap<String, String>,
    result_data: HashMap<String, String>,
    timeout_detector: TimeoutDetector,
    layer_timing: LayerTiming,
}

impl ObservabilityTracker {
    pub fn add_param<T: ToString>(&mut self, key: &str, value: T) {
        self.params.insert(key.to_string(), value.to_string());
    }

    pub fn add_result<T: ToString>(&mut self, key: &str, value: T) {
        self.result_data.insert(key.to_string(), value.to_string());
    }

    pub fn finish(self, err: Option<&anyhow::Error>) {
        let duration = self.start.elapsed();
        self.timeout_detector
            .check_timeout(false, false);
        self.layer_timing.finish_without_duration(err);
        metrics::record_layer_duration(
            &self.layer,
            &self.usecase,
            &self.operation,
            duration.as_secs_f64(),
            err.is_none(),
        );
        if self.layer == "service" {
            metrics::record_business_operation(
                &self.usecase,
                &self.operation,
                duration,
                err.is_none(),
            );
        }
        let mut attrs = Vec::new();
        for (k, v) in &self.params {
            attrs.push((k.clone(), v.clone()));
        }
        for (k, v) in &self.result_data {
            attrs.push((format!("result_{}", k), v.clone()));
        }
        metrics::record_structured_log_metric(&attrs);
    }
}

pub fn start_operation(usecase: &str, operation: &str, layer: &str) -> ObservabilityTracker {
    ObservabilityTracker {
        usecase: usecase.to_string(),
        operation: operation.to_string(),
        layer: layer.to_string(),
        start: Instant::now(),
        params: HashMap::new(),
        result_data: HashMap::new(),
        timeout_detector: TimeoutDetector::new(usecase, operation),
        layer_timing: LayerTiming::new(usecase, operation, layer),
    }
}
