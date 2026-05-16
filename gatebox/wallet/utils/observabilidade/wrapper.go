
package observabilidade

import (
	"context"
	"time"
)

// HandlerObservability - wrapper simplificado para handlers
type HandlerObservability struct {
	usecase string
}

// NewHandlerObservability cria novo observador para handler
func NewHandlerObservability(usecase string) *HandlerObservability {
	return &HandlerObservability{usecase: usecase}
}

// Track inicia rastreamento de uma operação do handler
func (ho *HandlerObservability) Track(ctx context.Context, operation string) *ObservabilityTracker {
	return StartOperation(ctx, ho.usecase, operation, "handler")
}

// ServiceObservability - wrapper para services
type ServiceObservability struct {
	usecase string
}

// NewServiceObservability cria novo observador para service
func NewServiceObservability(usecase string) *ServiceObservability {
	return &ServiceObservability{usecase: usecase}
}

// Track inicia rastreamento de uma operação do service
func (so *ServiceObservability) Track(ctx context.Context, operation string) *ObservabilityTracker {
	return StartOperation(ctx, so.usecase, operation, "service")
}

// RepositoryObservability - wrapper para repositories
type RepositoryObservability struct {
	usecase string
}

// NewRepositoryObservability cria novo observador para repository
func NewRepositoryObservability(usecase string) *RepositoryObservability {
	return &RepositoryObservability{usecase: usecase}
}

// Track inicia rastreamento de uma operação do repository
func (ro *RepositoryObservability) Track(ctx context.Context, operation string) *ObservabilityTracker {
	return StartOperation(ctx, ro.usecase, operation, "repository")
}

// TrackQuery rastreia especificamente queries de banco (mais específico para repositories)
func (ro *RepositoryObservability) TrackQuery(ctx context.Context, operation, table string) *QueryTracker {
	tracker := StartOperation(ctx, ro.usecase, operation+"_"+table, "repository")

	return &QueryTracker{
		ObservabilityTracker: tracker,
		operation:            operation,
		table:                table,
	}
}

// QueryTracker - especializado para queries de banco
type QueryTracker struct {
	*ObservabilityTracker
	operation string
	table     string
}

// Finish finaliza com métricas específicas de DB
func (qt *QueryTracker) Finish(errPtr *error) {
	duration := time.Since(qt.start)
	var err error
	if errPtr != nil {
		err = *errPtr
	}

	// Registrar métricas de DB antes do finish geral
	RecordDBQuery(qt.ctx, qt.usecase, qt.operation, qt.table, duration, err)

	// Chamar finish do tracker geral
	qt.ObservabilityTracker.Finish(errPtr)
}


