
package observabilidade

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/prometheus/client_golang/prometheus/promhttp"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/prometheus"
	"go.opentelemetry.io/otel/metric"
	sdkmetric "go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/resource"
	semconv "go.opentelemetry.io/otel/semconv/v1.17.0"
)

var (
	Meter metric.Meter
)

// InitObservability inicializa OpenTelemetry com Prometheus
func InitObservability(serviceName, serviceVersion string) error {
	// Criar resource com configuração simplificada para evitar conflitos
	res, err := resource.New(
		context.Background(),
		resource.WithAttributes(
			semconv.ServiceName(serviceName),
			semconv.ServiceVersion(serviceVersion),
		),
	)
	if err != nil {
		return fmt.Errorf("failed to create resource: %w", err)
	}

	// Configurar Prometheus exporter
	promExporter, err := prometheus.New()
	if err != nil {
		return fmt.Errorf("failed to create prometheus exporter: %w", err)
	}

	// Criar metric provider
	provider := sdkmetric.NewMeterProvider(
		sdkmetric.WithResource(res),
		sdkmetric.WithReader(promExporter),
	)

	// Definir como global
	otel.SetMeterProvider(provider)

	// Criar meter global
	Meter = provider.Meter(serviceName)

	// Inicializar métricas básicas
	if err := initMetrics(); err != nil {
		return fmt.Errorf("failed to initialize metrics: %w", err)
	}

	// Inicializar métricas por camada
	if err := initLayerMetrics(); err != nil {
		return fmt.Errorf("failed to initialize layer metrics: %w", err)
	}

	// Inicializar métricas de contexto
	if err := initContextMetrics(); err != nil {
		return fmt.Errorf("failed to initialize context metrics: %w", err)
	}

	// Inicializar métricas de sistema
	go startSystemMetrics()

	log.Println("✅ Observabilidade completa inicializada com sucesso")
	return nil
}

// StartMetricsServer inicia o servidor de métricas
func StartMetricsServer(port string) {
	http.Handle("/metrics", promhttp.Handler())

	server := &http.Server{
		Addr:         ":" + port,
		ReadTimeout:  5 * time.Second,
		WriteTimeout: 10 * time.Second,
	}

	log.Printf("🔍 Servidor de métricas iniciado na porta %s", port)
	log.Printf("📊 Métricas disponíveis em: http://localhost:%s/metrics", port)

	if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
		log.Fatalf("Erro ao iniciar servidor de métricas: %v", err)
	}
}


