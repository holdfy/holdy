package acquirersHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   acquirersSV "palm-pay/app/acquirers/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterAcquirersHTTPEndpoints(router *echo.Group, uc acquirersSV.AcquirersServiceIF, log logger.Logger) {
	h := NewAcquirersHandler(uc, log)
	acquirersGroup := router.Group("/acquirers", observabilidade.EnhancedHTTPMetricsMiddleware("acquirers"))
	{
		acquirersGroup.GET("/acquirers", h.GetAcquirers)
		acquirersGroup.GET("/acquirers/:id", h.GetAcquirersById)
		acquirersGroup.GET("/acquirers/acquirercode/:acquirercode", h.GetAcquirersByAcquirerCode)
		acquirersGroup.POST("/acquirers", h.InsertAcquirers)
		acquirersGroup.PUT("/acquirers/:id", h.UpdateAcquirers)
		acquirersGroup.DELETE("/acquirers/:id", h.DeleteAcquirersById)
	}
}

