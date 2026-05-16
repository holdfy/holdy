
package observabilidade

import (
	"context"
	"fmt"
	"strings"
	"time"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
)

// ObservabilityTracker rastreia uma operação completa com defer
type ObservabilityTracker struct {
	ctx       context.Context
	usecase   string
	operation string
	layer     string
	start     time.Time

	// Para capturar dados durante a execução
	params     map[string]interface{}
	resultData map[string]interface{}

	// Detectores específicos
	timeoutDetector *TimeoutDetector
	layerTiming     *LayerTiming
}

// StartOperation inicia o rastreamento de uma operação
func StartOperation(ctx context.Context, usecase, operation, layer string) *ObservabilityTracker {
	tracker := &ObservabilityTracker{
		ctx:             ctx,
		usecase:         usecase,
		operation:       operation,
		layer:           layer,
		start:           time.Now(),
		params:          make(map[string]interface{}),
		resultData:      make(map[string]interface{}),
		timeoutDetector: NewTimeoutDetector(usecase, operation),
		layerTiming:     NewLayerTiming(usecase, operation, layer),
	}

	return tracker
}

// AddParam adiciona parâmetros para logging (opcional)
func (ot *ObservabilityTracker) AddParam(key string, value interface{}) {
	ot.params[key] = value
}

// AddResult adiciona dados do resultado (opcional)
func (ot *ObservabilityTracker) AddResult(key string, value interface{}) {
	ot.resultData[key] = value
}

// Finish finaliza o rastreamento - DEVE ser chamado com defer
func (ot *ObservabilityTracker) Finish(errPtr *error) {
	duration := time.Since(ot.start)
	var err error
	if errPtr != nil {
		err = *errPtr
	}

	// 1. Verificar timeout
	ot.timeoutDetector.CheckTimeout(ot.ctx)

	// 2. Finalizar timing da camada
	ot.layerTiming.Finish(ot.ctx, err)

	// 3. Registrar métricas específicas por camada
	ot.recordLayerMetrics(duration, err)

	// 4. Registrar métricas de negócio se for service layer
	if ot.layer == "service" {
		RecordBusinessOperation(ot.ctx, ot.usecase, ot.operation, duration, err == nil)
	}

	// 5. Log estruturado automático
	ot.recordStructuredLog(duration, err)
}

// recordLayerMetrics registra métricas específicas da camada
func (ot *ObservabilityTracker) recordLayerMetrics(duration time.Duration, err error) {
	attrs := []attribute.KeyValue{
		attribute.String("usecase", ot.usecase),
		attribute.String("operation", ot.operation),
		attribute.String("layer", ot.layer),
		attribute.Bool("success", err == nil),
	}

	// Adicionar parâmetros como atributos (limitado)
	for key, value := range ot.params {
		if len(attrs) < 10 { // Limitar atributos
			attrs = append(attrs, attribute.String("param_"+key, toString(value)))
		}
	}

	// Registrar métricas baseadas na camada
	switch ot.layer {
	case "handler":
		if handlerDuration != nil {
			handlerDuration.Record(ot.ctx, duration.Seconds(), metric.WithAttributes(attrs...))
		}

		// Métricas específicas do handler
		if err != nil {
			if isValidationError(err) {
				handlerValidationErrors.Add(ot.ctx, 1, metric.WithAttributes(attrs...))
			} else {
				handlerServiceErrors.Add(ot.ctx, 1, metric.WithAttributes(attrs...))
			}
		}

	case "service":
		if serviceDuration != nil {
			serviceDuration.Record(ot.ctx, duration.Seconds(), metric.WithAttributes(attrs...))
		}

	case "repository":
		if repositoryDuration != nil {
			repositoryDuration.Record(ot.ctx, duration.Seconds(), metric.WithAttributes(attrs...))
		}
	}

	// Registrar operação geral
	if handlerOperations != nil {
		status := "success"
		if err != nil {
			status = "error"
		}
		attrs = append(attrs, attribute.String("status", status))
		handlerOperations.Add(ot.ctx, 1, metric.WithAttributes(attrs...))
	}
}

// recordStructuredLog cria log estruturado automático
func (ot *ObservabilityTracker) recordStructuredLog(duration time.Duration, err error) {
	logData := map[string]interface{}{
		"usecase":     ot.usecase,
		"operation":   ot.operation,
		"layer":       ot.layer,
		"duration_ms": duration.Milliseconds(),
		"success":     err == nil,
		"timestamp":   time.Now().Format(time.RFC3339),
	}

	// Adicionar request ID se existir
	if requestID := GetRequestIDFromContext(ot.ctx); requestID != "unknown" {
		logData["request_id"] = requestID
	}

	// Adicionar parâmetros
	if len(ot.params) > 0 {
		logData["params"] = ot.params
	}

	// Adicionar resultado
	if len(ot.resultData) > 0 {
		logData["result"] = ot.resultData
	}

	// Adicionar erro se existir
	if err != nil {
		logData["error"] = err.Error()
		logData["error_type"] = getErrorType(err)
	}

	// Registrar como métrica de log estruturado
	recordStructuredLogMetric(ot.ctx, logData)
}

// Helpers
func toString(value interface{}) string {
	if value == nil {
		return "nil"
	}
	return fmt.Sprintf("%v", value)
}

func isValidationError(err error) bool {
	// Detectar se é erro de validação baseado na mensagem
	errStr := err.Error()
	return strings.Contains(errStr, "validation") ||
		strings.Contains(errStr, "invalid") ||
		strings.Contains(errStr, "required") ||
		strings.Contains(errStr, "Bad Request")
}

func getErrorType(err error) string {
	errStr := err.Error()

	if strings.Contains(errStr, "timeout") || strings.Contains(errStr, "deadline") {
		return "timeout"
	}
	if strings.Contains(errStr, "connection") || strings.Contains(errStr, "network") {
		return "network"
	}
	if isValidationError(err) {
		return "validation"
	}
	if strings.Contains(errStr, "not found") {
		return "not_found"
	}

	return "internal"
}

func recordStructuredLogMetric(ctx context.Context, logData map[string]interface{}) {
	// Converter para atributos OpenTelemetry
	attrs := []attribute.KeyValue{}

	for key, value := range logData {
		if len(attrs) < 15 { // Limitar atributos
			attrs = append(attrs, attribute.String(key, toString(value)))
		}
	}

	// Registrar como métrica
	if businessOperationsTotal != nil {
		businessOperationsTotal.Add(ctx, 1, metric.WithAttributes(attrs...))
	}
}

