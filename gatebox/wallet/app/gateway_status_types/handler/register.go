package gateway_status_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   gateway_status_typesSV "palm-pay/app/gateway_status_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterGateway_status_typesHTTPEndpoints(router *echo.Group, uc gateway_status_typesSV.Gateway_status_typesServiceIF, log logger.Logger) {
	h := NewGateway_status_typesHandler(uc, log)
	gateway_status_typesGroup := router.Group("/gateway_status_types", observabilidade.EnhancedHTTPMetricsMiddleware("gateway_status_types"))
	{
		gateway_status_typesGroup.GET("/gateway_status_types", h.GetGateway_status_types)
		gateway_status_typesGroup.GET("/gateway_status_types/:id", h.GetGateway_status_typesById)
		gateway_status_typesGroup.GET("/gateway_status_types/statuscode/:statuscode", h.GetGateway_status_typesByStatusCode)
		gateway_status_typesGroup.POST("/gateway_status_types", h.InsertGateway_status_types)
		gateway_status_typesGroup.PUT("/gateway_status_types/:id", h.UpdateGateway_status_types)
		gateway_status_typesGroup.DELETE("/gateway_status_types/:id", h.DeleteGateway_status_typesById)
	}
}

