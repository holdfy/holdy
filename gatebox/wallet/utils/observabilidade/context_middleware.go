
package observabilidade

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"time"

	"github.com/labstack/echo/v4"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
)

var (
	// Métricas de contexto
	requestsWithoutID  metric.Int64Counter
	requestIDGenerated metric.Int64Counter
)

// initContextMetrics inicializa métricas de contexto
func initContextMetrics() error {
	var err error

	requestsWithoutID, err = Meter.Int64Counter(
		"requests_without_id_total",
		metric.WithDescription("Total number of requests without request ID"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	requestIDGenerated, err = Meter.Int64Counter(
		"request_id_generated_total",
		metric.WithDescription("Total number of generated request IDs"),
		metric.WithUnit("1"),
	)
	if err != nil {
		return err
	}

	return nil
}

// generateRequestID gera um ID único para a requisição
func generateRequestID() string {
	bytes := make([]byte, 8)
	if _, err := rand.Read(bytes); err != nil {
		// Fallback para timestamp se falhar
		return hex.EncodeToString([]byte(time.Now().Format("20060102150405")))
	}
	return hex.EncodeToString(bytes)
}

// RequestIDMiddleware adiciona Request ID a todas as requisições
func RequestIDMiddleware() echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			// Verificar se já existe um Request ID no header
			requestID := c.Request().Header.Get("X-Request-ID")

			if requestID == "" {
				// Gerar novo Request ID
				requestID = generateRequestID()

				// Registrar métrica
				if requestIDGenerated != nil {
					requestIDGenerated.Add(c.Request().Context(), 1,
						metric.WithAttributes(
							attribute.String("generated", "true"),
						))
				}
			} else {
				// Request ID veio do cliente
				if requestIDGenerated != nil {
					requestIDGenerated.Add(c.Request().Context(), 1,
						metric.WithAttributes(
							attribute.String("generated", "false"),
						))
				}
			}

			// Adicionar Request ID ao contexto
			ctx := context.WithValue(c.Request().Context(), "request_id", requestID)
			c.SetRequest(c.Request().WithContext(ctx))

			// Adicionar ao response header
			c.Response().Header().Set("X-Request-ID", requestID)

			// Adicionar ao contexto do Echo para facilitar acesso
			c.Set("request_id", requestID)

			return next(c)
		}
	}
}

// StructuredLoggingMiddleware adiciona logging estruturado
func StructuredLoggingMiddleware(usecase string) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			start := time.Now()

			// Capturar informações da requisição
			req := c.Request()
			requestID, _ := c.Get("request_id").(string)

			// Log de entrada da requisição
			logAttrs := []attribute.KeyValue{
				attribute.String("usecase", usecase),
				attribute.String("request_id", requestID),
				attribute.String("method", req.Method),
				attribute.String("path", c.Path()),
				attribute.String("real_ip", c.RealIP()),
				attribute.String("user_agent", req.UserAgent()),
			}

			// Processar requisição
			err := next(c)

			// Calcular duração
			duration := time.Since(start)

			// Log de saída da requisição
			responseAttrs := append(logAttrs,
				attribute.Int("status_code", c.Response().Status),
				attribute.Int64("response_size", c.Response().Size),
				attribute.Float64("duration_seconds", duration.Seconds()),
			)

			// Registrar métrica de log estruturado
			RecordStructuredLog(c.Request().Context(), responseAttrs)

			return err
		}
	}
}

// TimeoutMiddleware adiciona timeout padrão se não existir
func TimeoutMiddleware(timeout time.Duration) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			ctx := c.Request().Context()

			// Verificar se já tem deadline
			if _, hasDeadline := ctx.Deadline(); !hasDeadline {
				var cancel context.CancelFunc
				ctx, cancel = context.WithTimeout(ctx, timeout)
				defer cancel()
				c.SetRequest(c.Request().WithContext(ctx))
			}

			return next(c)
		}
	}
}

// RecordStructuredLog registra log estruturado como métrica
func RecordStructuredLog(ctx context.Context, attrs []attribute.KeyValue) {
	// Esta função pode ser expandida para enviar logs para sistemas externos
	// Por agora, apenas registra como métrica
	if len(attrs) > 0 {
		// Encontrar status code para determinar se foi erro
		var statusCode int
		for _, attr := range attrs {
			if attr.Key == "status_code" {
				statusCode = int(attr.Value.AsInt64())
				break
			}
		}

		// Criar métrica baseada no status
		if statusCode >= 400 && requestsWithoutID != nil {
			requestsWithoutID.Add(ctx, 1, metric.WithAttributes(attrs...))
		}
	}
}

// GetRequestIDFromContext obtém Request ID do contexto
func GetRequestIDFromContext(ctx context.Context) string {
	if requestID, ok := ctx.Value("request_id").(string); ok {
		return requestID
	}
	return "unknown"
}

