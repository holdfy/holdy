
package observabilidade

import (
	"context"
	"time"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
)

var (
	// Métricas por Camada
	handlerDuration    metric.Float64Histogram
	serviceDuration    metric.Float64Histogram
	repositoryDuration metric.Float64Histogram

	// Métricas de Timeout
	timeoutOperations  metric.Int64Counter
	canceledOperations metric.Int64Counter

	// Métricas de Performance
	slowOperations metric.Int64Counter

	// Métricas específicas do handler
	handlerValidationErrors metric.Int64Counter
	handlerServiceErrors    metric.Int64Counter
	handlerOperations       metric.Int64Counter
	handlerCriticalOps      metric.Int64Counter
	handlerNotFound         metric.Int64Counter
)

// initLayerMetrics inicializa métricas específicas por camada
func initLayerMetrics() error {
	var err error

	// Handler Layer Metrics
	handlerDuration, err = Meter.Float64Histogram(
		"handler_duration_seconds",
		metric.WithDescription("Time spent in handler layer"),
		metric.WithUnit("s"),
	)
	if err != nil {
		return err
	}

	// Service Layer Metrics
	serviceDuration, err = Meter.Float64Histogram(
		"service_duration_seconds",
		metric.WithDescription("Time spent in service layer"),
		metric.WithUnit("s"),
	)
	if err != nil {
		return err
	}

	// Repository Layer Metrics
	repositoryDuration, err = Meter.Float64Histogram(
		"repository_duration_seconds",
		metric.WithDescription("Time spent in repository layer"),
		metric.WithUnit("s"),
	)
	if err != nil {
		return err
	}

	// Timeout Metrics
	timeoutOperations, err = Meter.Int64Counter(
		"timeout_operations_total",
		metric.WithDescription("Total number of operations that timed out"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	canceledOperations, err = Meter.Int64Counter(
		"canceled_operations_total",
		metric.WithDescription("Total number of canceled operations"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	// Performance Metrics
	slowOperations, err = Meter.Int64Counter(
		"slow_operations_total",
		metric.WithDescription("Total number of slow operations (>threshold)"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	// Handler specific metrics
	handlerValidationErrors, err = Meter.Int64Counter(
		"handler_validation_errors_total",
		metric.WithDescription("Total number of validation errors in handler"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	handlerServiceErrors, err = Meter.Int64Counter(
		"handler_service_errors_total",
		metric.WithDescription("Total number of service errors in handler"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	handlerOperations, err = Meter.Int64Counter(
		"handler_operations_total",
		metric.WithDescription("Total number of handler operations"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	handlerCriticalOps, err = Meter.Int64Counter(
		"handler_critical_operations_total",
		metric.WithDescription("Total number of critical operations (create/update/delete)"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	handlerNotFound, err = Meter.Int64Counter(
		"handler_not_found_total",
		metric.WithDescription("Total number of not found cases"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	return nil
}

// LayerTiming estrutura para medir tempo por camada
type LayerTiming struct {
	usecase   string
	operation string
	layer     string
	start     time.Time
}

// NewLayerTiming cria um novo timer para uma camada
func NewLayerTiming(usecase, operation, layer string) *LayerTiming {
	return &LayerTiming{
		usecase:   usecase,
		operation: operation,
		layer:     layer,
		start:     time.Now(),
	}
}

// Finish finaliza o timing e registra a métrica
func (lt *LayerTiming) Finish(ctx context.Context, err error) {
	duration := time.Since(lt.start)

	attrs := []attribute.KeyValue{
		attribute.String("usecase", lt.usecase),
		attribute.String("operation", lt.operation),
		attribute.String("layer", lt.layer),
		attribute.Bool("success", err == nil),
	}

	// Registrar duração baseada na camada
	switch lt.layer {
	case "handler":
		if handlerDuration != nil {
			handlerDuration.Record(ctx, duration.Seconds(), metric.WithAttributes(attrs...))
		}
	case "service":
		if serviceDuration != nil {
			serviceDuration.Record(ctx, duration.Seconds(), metric.WithAttributes(attrs...))
		}
	case "repository":
		if repositoryDuration != nil {
			repositoryDuration.Record(ctx, duration.Seconds(), metric.WithAttributes(attrs...))
		}
	}

	// Detectar operações lentas (>1s para handler, >500ms para service, >200ms para repository)
	var threshold time.Duration
	switch lt.layer {
	case "handler":
		threshold = 1 * time.Second
	case "service":
		threshold = 500 * time.Millisecond
	case "repository":
		threshold = 200 * time.Millisecond
	}

	if duration > threshold && slowOperations != nil {
		slowOperations.Add(ctx, 1, metric.WithAttributes(attrs...))
	}

	// Detectar timeouts e cancelamentos
	if ctx.Err() == context.DeadlineExceeded && timeoutOperations != nil {
		timeoutOperations.Add(ctx, 1, metric.WithAttributes(attrs...))
	} else if ctx.Err() == context.Canceled && canceledOperations != nil {
		canceledOperations.Add(ctx, 1, metric.WithAttributes(attrs...))
	}
}

// TimeoutDetector estrutura para detectar timeouts
type TimeoutDetector struct {
	usecase   string
	operation string
	start     time.Time
}

// NewTimeoutDetector cria um novo detector de timeout
func NewTimeoutDetector(usecase, operation string) *TimeoutDetector {
	return &TimeoutDetector{
		usecase:   usecase,
		operation: operation,
		start:     time.Now(),
	}
}

// CheckTimeout verifica se houve timeout
func (td *TimeoutDetector) CheckTimeout(ctx context.Context) {
	if ctx.Err() != nil {
		attrs := []attribute.KeyValue{
			attribute.String("usecase", td.usecase),
			attribute.String("operation", td.operation),
			attribute.String("error_type", ctx.Err().Error()),
		}

		if ctx.Err() == context.DeadlineExceeded && timeoutOperations != nil {
			timeoutOperations.Add(ctx, 1, metric.WithAttributes(attrs...))
		} else if ctx.Err() == context.Canceled && canceledOperations != nil {
			canceledOperations.Add(ctx, 1, metric.WithAttributes(attrs...))
		}
	}
}


