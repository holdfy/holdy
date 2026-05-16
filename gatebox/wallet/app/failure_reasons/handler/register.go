package failure_reasonsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   failure_reasonsSV "palm-pay/app/failure_reasons/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterFailure_reasonsHTTPEndpoints(router *echo.Group, uc failure_reasonsSV.Failure_reasonsServiceIF, log logger.Logger) {
	h := NewFailure_reasonsHandler(uc, log)
	failure_reasonsGroup := router.Group("/failure_reasons", observabilidade.EnhancedHTTPMetricsMiddleware("failure_reasons"))
	{
		failure_reasonsGroup.GET("/failure_reasons", h.GetFailure_reasons)
		failure_reasonsGroup.GET("/failure_reasons/:id", h.GetFailure_reasonsById)
		failure_reasonsGroup.GET("/failure_reasons/reasoncode/:reasoncode", h.GetFailure_reasonsByReasonCode)
		failure_reasonsGroup.POST("/failure_reasons", h.InsertFailure_reasons)
		failure_reasonsGroup.PUT("/failure_reasons/:id", h.UpdateFailure_reasons)
		failure_reasonsGroup.DELETE("/failure_reasons/:id", h.DeleteFailure_reasonsById)
	}
}

