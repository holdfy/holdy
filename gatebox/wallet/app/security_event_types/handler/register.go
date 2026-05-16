package security_event_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   security_event_typesSV "palm-pay/app/security_event_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterSecurity_event_typesHTTPEndpoints(router *echo.Group, uc security_event_typesSV.Security_event_typesServiceIF, log logger.Logger) {
	h := NewSecurity_event_typesHandler(uc, log)
	security_event_typesGroup := router.Group("/security_event_types", observabilidade.EnhancedHTTPMetricsMiddleware("security_event_types"))
	{
		security_event_typesGroup.GET("/security_event_types", h.GetSecurity_event_types)
		security_event_typesGroup.GET("/security_event_types/:id", h.GetSecurity_event_typesById)
		security_event_typesGroup.GET("/security_event_types/typecode/:typecode", h.GetSecurity_event_typesByTypeCode)
		security_event_typesGroup.POST("/security_event_types", h.InsertSecurity_event_types)
		security_event_typesGroup.PUT("/security_event_types/:id", h.UpdateSecurity_event_types)
		security_event_typesGroup.DELETE("/security_event_types/:id", h.DeleteSecurity_event_typesById)
	}
}

