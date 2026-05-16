
package observabilidade

import (
	"context"
	"time"

	"github.com/labstack/echo/v4"
)

// HTTPMetricsMiddleware middleware básico para coletar métricas HTTP
func HTTPMetricsMiddleware(usecase string) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			start := time.Now()

			// Processar requisição
			err := next(c)

			// Registrar métricas
			duration := time.Since(start)
			method := c.Request().Method
			path := c.Path()
			statusCode := c.Response().Status

			RecordHTTPRequest(
				c.Request().Context(),
				usecase,
				method,
				path,
				duration,
				statusCode,
			)

			return err
		}
	}
}

// EnhancedHTTPMetricsMiddleware middleware avançado para métricas HTTP
func EnhancedHTTPMetricsMiddleware(usecase string) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			// Criar context com timeout se não existir
			ctx := c.Request().Context()
			if _, hasDeadline := ctx.Deadline(); !hasDeadline {
				var cancel context.CancelFunc
				ctx, cancel = context.WithTimeout(ctx, 30*time.Second)
				defer cancel()
				c.SetRequest(c.Request().WithContext(ctx))
			}

			// Timing da camada handler
			handlerTiming := NewLayerTiming(usecase, c.Path(), "handler")

			// Detector de timeout
			timeoutDetector := NewTimeoutDetector(usecase, c.Path())

			start := time.Now()

			// Processar requisição
			err := next(c)

			// Verificar timeout
			timeoutDetector.CheckTimeout(ctx)

			// Finalizar timing do handler
			handlerTiming.Finish(ctx, err)

			// Registrar métricas HTTP originais
			duration := time.Since(start)
			method := c.Request().Method
			path := c.Path()
			statusCode := c.Response().Status

			RecordHTTPRequest(
				ctx,
				usecase,
				method,
				path,
				duration,
				statusCode,
			)

			return err
		}
	}
}

