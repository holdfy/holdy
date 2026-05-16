package gatewaysHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   gatewaysSV "palm-pay/app/gateways/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterGatewaysHTTPEndpoints(router *echo.Group, uc gatewaysSV.GatewaysServiceIF, log logger.Logger) {
	h := NewGatewaysHandler(uc, log)
	gatewaysGroup := router.Group("/gateways", observabilidade.EnhancedHTTPMetricsMiddleware("gateways"))
	{
		gatewaysGroup.GET("/gateways", h.GetGateways)
		gatewaysGroup.GET("/gateways/:id", h.GetGatewaysById)
		gatewaysGroup.GET("/gateways/gatewaycode/:gatewaycode", h.GetGatewaysByGatewayCode)
		gatewaysGroup.POST("/gateways", h.InsertGateways)
		gatewaysGroup.PUT("/gateways/:id", h.UpdateGateways)
		gatewaysGroup.DELETE("/gateways/:id", h.DeleteGatewaysById)
	}
}

