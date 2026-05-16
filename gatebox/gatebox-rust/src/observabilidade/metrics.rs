// Converted from gateboxgo/observabilidade/metrics.go and utils/observabilidade/layer_metrics.go
use anyhow::Result;
use prometheus::{register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec};
use std::sync::OnceLock;
use std::time::Duration;

static HTTP_REQUESTS_TOTAL: OnceLock<IntCounterVec> = OnceLock::new();
static HTTP_REQUEST_DURATION: OnceLock<HistogramVec> = OnceLock::new();
static HTTP_ERRORS_TOTAL: OnceLock<IntCounterVec> = OnceLock::new();
static DB_QUERIES_TOTAL: OnceLock<IntCounterVec> = OnceLock::new();
static DB_QUERY_DURATION: OnceLock<HistogramVec> = OnceLock::new();
static DB_QUERY_ERRORS: OnceLock<IntCounterVec> = OnceLock::new();
static BUSINESS_OPERATIONS_TOTAL: OnceLock<IntCounterVec> = OnceLock::new();
static BUSINESS_OPERATION_DURATION: OnceLock<HistogramVec> = OnceLock::new();
static HANDLER_DURATION: OnceLock<HistogramVec> = OnceLock::new();
static SERVICE_DURATION: OnceLock<HistogramVec> = OnceLock::new();
static REPOSITORY_DURATION: OnceLock<HistogramVec> = OnceLock::new();
static TIMEOUT_OPERATIONS: OnceLock<IntCounterVec> = OnceLock::new();
static SLOW_OPERATIONS: OnceLock<IntCounterVec> = OnceLock::new();

pub fn registry() -> &'static prometheus::Registry {
    prometheus::default_registry()
}

pub fn register_metrics(_service_name: &str) -> Result<()> {
    let http_requests = register_int_counter_vec!(
        "http_requests_total",
        "Total number of HTTP requests",
        &["usecase", "method", "endpoint", "status_code"]
    )?;
    let _ = HTTP_REQUESTS_TOTAL.set(http_requests);

    let http_duration = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["usecase", "method", "endpoint", "status_code"]
    )?;
    let _ = HTTP_REQUEST_DURATION.set(http_duration);

    let http_errors = register_int_counter_vec!(
        "http_errors_total",
        "Total number of HTTP errors",
        &["usecase", "method", "endpoint", "status_code"]
    )?;
    let _ = HTTP_ERRORS_TOTAL.set(http_errors);

    let db_queries = register_int_counter_vec!(
        "db_queries_total",
        "Total number of database queries",
        &["usecase", "operation", "table"]
    )?;
    let _ = DB_QUERIES_TOTAL.set(db_queries);

    let db_duration = register_histogram_vec!(
        "db_query_duration_seconds",
        "Database query duration in seconds",
        &["usecase", "operation", "table"]
    )?;
    let _ = DB_QUERY_DURATION.set(db_duration);

    let db_errors = register_int_counter_vec!(
        "db_query_errors_total",
        "Total number of database query errors",
        &["usecase", "operation", "table"]
    )?;
    let _ = DB_QUERY_ERRORS.set(db_errors);

    let biz_total = register_int_counter_vec!(
        "business_operations_total",
        "Total number of business operations",
        &["usecase", "operation", "success"]
    )?;
    let _ = BUSINESS_OPERATIONS_TOTAL.set(biz_total);

    let biz_duration = register_histogram_vec!(
        "business_operation_duration_seconds",
        "Business operation duration in seconds",
        &["usecase", "operation", "success"]
    )?;
    let _ = BUSINESS_OPERATION_DURATION.set(biz_duration);

    let handler_dur = register_histogram_vec!(
        "handler_duration_seconds",
        "Time spent in handler layer",
        &["usecase", "operation", "layer", "success"]
    )?;
    let _ = HANDLER_DURATION.set(handler_dur);

    let service_dur = register_histogram_vec!(
        "service_duration_seconds",
        "Time spent in service layer",
        &["usecase", "operation", "layer", "success"]
    )?;
    let _ = SERVICE_DURATION.set(service_dur);

    let repo_dur = register_histogram_vec!(
        "repository_duration_seconds",
        "Time spent in repository layer",
        &["usecase", "operation", "layer", "success"]
    )?;
    let _ = REPOSITORY_DURATION.set(repo_dur);

    let timeout = register_int_counter_vec!(
        "timeout_operations_total",
        "Total number of operations that timed out",
        &["usecase", "operation"]
    )?;
    let _ = TIMEOUT_OPERATIONS.set(timeout);

    let slow = register_int_counter_vec!(
        "slow_operations_total",
        "Total number of slow operations",
        &["usecase", "operation", "layer", "success"]
    )?;
    let _ = SLOW_OPERATIONS.set(slow);

    Ok(())
}

pub fn record_http_request(
    usecase: &str,
    method: &str,
    endpoint: &str,
    duration: Duration,
    status_code: u16,
) {
    let status = status_code.to_string();
    if let Some(c) = HTTP_REQUESTS_TOTAL.get() {
        c.with_label_values(&[usecase, method, endpoint, &status]).inc();
    }
    if let Some(h) = HTTP_REQUEST_DURATION.get() {
        h.with_label_values(&[usecase, method, endpoint, &status])
            .observe(duration.as_secs_f64());
    }
    if status_code >= 400 {
        if let Some(c) = HTTP_ERRORS_TOTAL.get() {
            c.with_label_values(&[usecase, method, endpoint, &status]).inc();
        }
    }
}

pub fn record_db_query(
    usecase: &str,
    operation: &str,
    table: &str,
    duration: Duration,
    err: Option<&anyhow::Error>,
) {
    let success = if err.is_some() { "false" } else { "true" };
    if let Some(c) = DB_QUERIES_TOTAL.get() {
        c.with_label_values(&[usecase, operation, table]).inc();
    }
    if let Some(h) = DB_QUERY_DURATION.get() {
        h.with_label_values(&[usecase, operation, table]).observe(duration.as_secs_f64());
    }
    if err.is_some() {
        if let Some(c) = DB_QUERY_ERRORS.get() {
            c.with_label_values(&[usecase, operation, table]).inc();
        }
    }
    let _ = success;
}

pub fn record_business_operation(
    usecase: &str,
    operation: &str,
    duration: Duration,
    success: bool,
) {
    let success_str = if success { "true" } else { "false" };
    if let Some(c) = BUSINESS_OPERATIONS_TOTAL.get() {
        c.with_label_values(&[usecase, operation, success_str]).inc();
    }
    if let Some(h) = BUSINESS_OPERATION_DURATION.get() {
        h.with_label_values(&[usecase, operation, success_str]).observe(duration.as_secs_f64());
    }
}

pub fn record_layer_duration(
    layer: &str,
    usecase: &str,
    operation: &str,
    duration_secs: f64,
    success: bool,
) {
    let success_str = if success { "true" } else { "false" };
    let labels = [usecase, operation, layer, success_str];
    match layer {
        "handler" => {
            if let Some(h) = HANDLER_DURATION.get() {
                h.with_label_values(&labels).observe(duration_secs);
            }
        }
        "service" => {
            if let Some(h) = SERVICE_DURATION.get() {
                h.with_label_values(&labels).observe(duration_secs);
            }
        }
        "repository" => {
            if let Some(h) = REPOSITORY_DURATION.get() {
                h.with_label_values(&labels).observe(duration_secs);
            }
        }
        _ => {}
    }
}

pub fn record_slow_operation(usecase: &str, operation: &str, layer: &str, success: bool) {
    let success_str = if success { "true" } else { "false" };
    if let Some(c) = SLOW_OPERATIONS.get() {
        c.with_label_values(&[usecase, operation, layer, success_str]).inc();
    }
}

pub fn record_timeout(usecase: &str, operation: &str) {
    if let Some(c) = TIMEOUT_OPERATIONS.get() {
        c.with_label_values(&[usecase, operation]).inc();
    }
}

pub fn record_structured_log_metric(_attrs: &[(String, String)]) {
    // Placeholder: could emit to a log metric counter
}
