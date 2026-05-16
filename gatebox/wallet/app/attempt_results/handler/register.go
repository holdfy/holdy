package attempt_resultsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   attempt_resultsSV "palm-pay/app/attempt_results/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterAttempt_resultsHTTPEndpoints(router *echo.Group, uc attempt_resultsSV.Attempt_resultsServiceIF, log logger.Logger) {
	h := NewAttempt_resultsHandler(uc, log)
	attempt_resultsGroup := router.Group("/attempt_results", observabilidade.EnhancedHTTPMetricsMiddleware("attempt_results"))
	{
		attempt_resultsGroup.GET("/attempt_results", h.GetAttempt_results)
		attempt_resultsGroup.GET("/attempt_results/:id", h.GetAttempt_resultsById)
		attempt_resultsGroup.GET("/attempt_results/resultcode/:resultcode", h.GetAttempt_resultsByResultCode)
		attempt_resultsGroup.POST("/attempt_results", h.InsertAttempt_results)
		attempt_resultsGroup.PUT("/attempt_results/:id", h.UpdateAttempt_results)
		attempt_resultsGroup.DELETE("/attempt_results/:id", h.DeleteAttempt_resultsById)
	}
}

