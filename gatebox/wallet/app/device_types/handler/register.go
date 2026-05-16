package device_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   device_typesSV "palm-pay/app/device_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterDevice_typesHTTPEndpoints(router *echo.Group, uc device_typesSV.Device_typesServiceIF, log logger.Logger) {
	h := NewDevice_typesHandler(uc, log)
	device_typesGroup := router.Group("/device_types", observabilidade.EnhancedHTTPMetricsMiddleware("device_types"))
	{
		device_typesGroup.GET("/device_types", h.GetDevice_types)
		device_typesGroup.GET("/device_types/:id", h.GetDevice_typesById)
		device_typesGroup.GET("/device_types/typecode/:typecode", h.GetDevice_typesByTypeCode)
		device_typesGroup.POST("/device_types", h.InsertDevice_types)
		device_typesGroup.PUT("/device_types/:id", h.UpdateDevice_types)
		device_typesGroup.DELETE("/device_types/:id", h.DeleteDevice_typesById)
	}
}

