// Converted from gateboxgo/utils/observabilidade/wrapper.go
use crate::observabilidade::metrics;
use crate::observabilidade::tracker::{start_operation, ObservabilityTracker};

pub struct HandlerObservability {
    usecase: String,
}

impl HandlerObservability {
    pub fn new(usecase: &str) -> Self {
        Self {
            usecase: usecase.to_string(),
        }
    }

    pub fn track(&self, operation: &str) -> ObservabilityTracker {
        start_operation(&self.usecase, operation, "handler")
    }
}

pub struct ServiceObservability {
    usecase: String,
}

impl ServiceObservability {
    pub fn new(usecase: &str) -> Self {
        Self {
            usecase: usecase.to_string(),
        }
    }

    pub fn track(&self, operation: &str) -> ObservabilityTracker {
        start_operation(&self.usecase, operation, "service")
    }
}

pub struct RepositoryObservability {
    usecase: String,
}

impl RepositoryObservability {
    pub fn new(usecase: &str) -> Self {
        Self {
            usecase: usecase.to_string(),
        }
    }

    pub fn track(&self, operation: &str) -> ObservabilityTracker {
        start_operation(&self.usecase, operation, "repository")
    }

    pub fn track_query(&self, operation: &str, table: &str) -> QueryTracker {
        let tracker = start_operation(&self.usecase, &format!("{}_{}", operation, table), "repository");
        QueryTracker {
            tracker,
            usecase: self.usecase.clone(),
            operation: operation.to_string(),
            table: table.to_string(),
        }
    }
}

pub struct QueryTracker {
    tracker: ObservabilityTracker,
    usecase: String,
    operation: String,
    table: String,
}

impl QueryTracker {
    pub fn finish(self, err: Option<&anyhow::Error>) {
        let duration = self.tracker.start.elapsed();
        metrics::record_db_query(
            &self.usecase,
            &self.operation,
            &self.table,
            duration,
            err,
        );
        self.tracker.finish(err);
    }
}
