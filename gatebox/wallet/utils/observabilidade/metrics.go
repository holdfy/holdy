
package observabilidade

import (
	"context"
	"runtime"
	"time"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
)

var (
	// HTTP Metrics
	httpRequestsTotal   metric.Int64Counter
	httpRequestDuration metric.Float64Histogram
	httpErrorsTotal     metric.Int64Counter

	// Database Metrics
	dbConnectionsActive metric.Int64UpDownCounter
	dbQueryDuration     metric.Float64Histogram
	dbQueriesTotal      metric.Int64Counter
	dbQueryErrors       metric.Int64Counter

	// System Metrics
	memoryUsage      metric.Int64UpDownCounter
	goroutinesActive metric.Int64UpDownCounter
	gcDuration       metric.Float64Histogram

	// Business Metrics
	businessOperationsTotal   metric.Int64Counter
	businessOperationDuration metric.Float64Histogram
)

// initMetrics inicializa todas as métricas básicas
func initMetrics() error {
	var err error

	// HTTP Metrics
	httpRequestsTotal, err = Meter.Int64Counter(
		"http_requests_total",
		metric.WithDescription("Total number of HTTP requests"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	httpRequestDuration, err = Meter.Float64Histogram(
		"http_request_duration_seconds",
		metric.WithDescription("HTTP request duration in seconds"),
		metric.WithUnit("s"),
	)
	if err != nil {
		return err
	}

	httpErrorsTotal, err = Meter.Int64Counter(
		"http_errors_total",
		metric.WithDescription("Total number of HTTP errors"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	// Database Metrics
	dbConnectionsActive, err = Meter.Int64UpDownCounter(
		"db_connections_active",
		metric.WithDescription("Number of active database connections"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	dbQueryDuration, err = Meter.Float64Histogram(
		"db_query_duration_seconds",
		metric.WithDescription("Database query duration in seconds"),
		metric.WithUnit("s"),
	)
	if err != nil {
		return err
	}

	dbQueriesTotal, err = Meter.Int64Counter(
		"db_queries_total",
		metric.WithDescription("Total number of database queries"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	dbQueryErrors, err = Meter.Int64Counter(
		"db_query_errors_total",
		metric.WithDescription("Total number of database query errors"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	// System Metrics
	memoryUsage, err = Meter.Int64UpDownCounter(
		"memory_usage_bytes",
		metric.WithDescription("Memory usage in bytes"),
		metric.WithUnit("bytes"),
	)
	if err != nil {
		return err
	}

	goroutinesActive, err = Meter.Int64UpDownCounter(
		"goroutines_active",
		metric.WithDescription("Number of active goroutines"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	gcDuration, err = Meter.Float64Histogram(
		"gc_duration_seconds",
		metric.WithDescription("Garbage collection duration in seconds"),
		metric.WithUnit("s"),
	)
	if err != nil {
		return err
	}

	// Business Metrics
	businessOperationsTotal, err = Meter.Int64Counter(
		"business_operations_total",
		metric.WithDescription("Total number of business operations"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	businessOperationDuration, err = Meter.Float64Histogram(
		"business_operation_duration_seconds",
		metric.WithDescription("Business operation duration in seconds"),
		metric.WithUnit("s"),
	)
	if err != nil {
		return err
	}

	return nil
}

// RecordHTTPRequest registra uma requisição HTTP
func RecordHTTPRequest(ctx context.Context, usecase, method, endpoint string, duration time.Duration, statusCode int) {
	attrs := []attribute.KeyValue{
		attribute.String("usecase", usecase),
		attribute.String("method", method),
		attribute.String("endpoint", endpoint),
		attribute.Int("status_code", statusCode),
	}

	if httpRequestsTotal != nil {
		httpRequestsTotal.Add(ctx, 1, metric.WithAttributes(attrs...))
	}
	if httpRequestDuration != nil {
		httpRequestDuration.Record(ctx, duration.Seconds(), metric.WithAttributes(attrs...))
	}

	if statusCode >= 400 && httpErrorsTotal != nil {
		httpErrorsTotal.Add(ctx, 1, metric.WithAttributes(attrs...))
	}
}

// RecordDBQuery registra uma query de banco de dados
func RecordDBQuery(ctx context.Context, usecase, operation, table string, duration time.Duration, err error) {
	attrs := []attribute.KeyValue{
		attribute.String("usecase", usecase),
		attribute.String("operation", operation),
		attribute.String("table", table),
	}

	if dbQueriesTotal != nil {
		dbQueriesTotal.Add(ctx, 1, metric.WithAttributes(attrs...))
	}
	if dbQueryDuration != nil {
		dbQueryDuration.Record(ctx, duration.Seconds(), metric.WithAttributes(attrs...))
	}

	if err != nil && dbQueryErrors != nil {
		dbQueryErrors.Add(ctx, 1, metric.WithAttributes(attrs...))
	}
}

// RecordBusinessOperation registra uma operação de negócio
func RecordBusinessOperation(ctx context.Context, usecase, operation string, duration time.Duration, success bool) {
	attrs := []attribute.KeyValue{
		attribute.String("usecase", usecase),
		attribute.String("operation", operation),
		attribute.Bool("success", success),
	}

	if businessOperationsTotal != nil {
		businessOperationsTotal.Add(ctx, 1, metric.WithAttributes(attrs...))
	}
	if businessOperationDuration != nil {
		businessOperationDuration.Record(ctx, duration.Seconds(), metric.WithAttributes(attrs...))
	}
}

// UpdateDBConnections atualiza o número de conexões ativas do banco
func UpdateDBConnections(ctx context.Context, usecase string, connections int) {
	attrs := []attribute.KeyValue{
		attribute.String("usecase", usecase),
	}
	if dbConnectionsActive != nil {
		dbConnectionsActive.Add(ctx, int64(connections), metric.WithAttributes(attrs...))
	}
}

// startSystemMetrics inicia a coleta de métricas do sistema
func startSystemMetrics() {
	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		var m runtime.MemStats
		runtime.ReadMemStats(&m)

		ctx := context.Background()
		attrs := []attribute.KeyValue{
			attribute.String("type", "heap"),
		}

		if memoryUsage != nil {
			memoryUsage.Add(ctx, int64(m.Alloc), metric.WithAttributes(attrs...))
		}
		if goroutinesActive != nil {
			goroutinesActive.Add(ctx, int64(runtime.NumGoroutine()), metric.WithAttributes(attrs...))
		}
	}
}

