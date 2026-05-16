package address_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   address_typesSV "palm-pay/app/address_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterAddress_typesHTTPEndpoints(router *echo.Group, uc address_typesSV.Address_typesServiceIF, log logger.Logger) {
	h := NewAddress_typesHandler(uc, log)
	address_typesGroup := router.Group("/address_types", observabilidade.EnhancedHTTPMetricsMiddleware("address_types"))
	{
		address_typesGroup.GET("/address_types", h.GetAddress_types)
		address_typesGroup.GET("/address_types/:id", h.GetAddress_typesById)
		address_typesGroup.GET("/address_types/typecode/:typecode", h.GetAddress_typesByTypeCode)
		address_typesGroup.POST("/address_types", h.InsertAddress_types)
		address_typesGroup.PUT("/address_types/:id", h.UpdateAddress_types)
		address_typesGroup.DELETE("/address_types/:id", h.DeleteAddress_typesById)
	}
}

