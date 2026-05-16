package restriction_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   restriction_typesSV "palm-pay/app/restriction_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterRestriction_typesHTTPEndpoints(router *echo.Group, uc restriction_typesSV.Restriction_typesServiceIF, log logger.Logger) {
	h := NewRestriction_typesHandler(uc, log)
	restriction_typesGroup := router.Group("/restriction_types", observabilidade.EnhancedHTTPMetricsMiddleware("restriction_types"))
	{
		restriction_typesGroup.GET("/restriction_types", h.GetRestriction_types)
		restriction_typesGroup.GET("/restriction_types/:id", h.GetRestriction_typesById)
		restriction_typesGroup.GET("/restriction_types/typecode/:typecode", h.GetRestriction_typesByTypeCode)
		restriction_typesGroup.POST("/restriction_types", h.InsertRestriction_types)
		restriction_typesGroup.PUT("/restriction_types/:id", h.UpdateRestriction_types)
		restriction_typesGroup.DELETE("/restriction_types/:id", h.DeleteRestriction_typesById)
	}
}

