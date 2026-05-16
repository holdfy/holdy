package biometric_attemptsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   biometric_attemptsSV "palm-pay/app/biometric_attempts/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterBiometric_attemptsHTTPEndpoints(router *echo.Group, uc biometric_attemptsSV.Biometric_attemptsServiceIF, log logger.Logger) {
	h := NewBiometric_attemptsHandler(uc, log)
	biometric_attemptsGroup := router.Group("/biometric_attempts", observabilidade.EnhancedHTTPMetricsMiddleware("biometric_attempts"))
	{
		biometric_attemptsGroup.GET("/biometric_attempts", h.GetBiometric_attempts)
		biometric_attemptsGroup.GET("/biometric_attempts/:id", h.GetBiometric_attemptsById)
		biometric_attemptsGroup.GET("/biometric_attempts/attemptcode/:attemptcode", h.GetBiometric_attemptsByAttemptCode)
		biometric_attemptsGroup.POST("/biometric_attempts", h.InsertBiometric_attempts)
		biometric_attemptsGroup.PUT("/biometric_attempts/:id", h.UpdateBiometric_attempts)
		biometric_attemptsGroup.DELETE("/biometric_attempts/:id", h.DeleteBiometric_attemptsById)
	}
}

