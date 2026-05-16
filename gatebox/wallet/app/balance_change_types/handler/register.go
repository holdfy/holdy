package balance_change_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   balance_change_typesSV "palm-pay/app/balance_change_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterBalance_change_typesHTTPEndpoints(router *echo.Group, uc balance_change_typesSV.Balance_change_typesServiceIF, log logger.Logger) {
	h := NewBalance_change_typesHandler(uc, log)
	balance_change_typesGroup := router.Group("/balance_change_types", observabilidade.EnhancedHTTPMetricsMiddleware("balance_change_types"))
	{
		balance_change_typesGroup.GET("/balance_change_types", h.GetBalance_change_types)
		balance_change_typesGroup.GET("/balance_change_types/:id", h.GetBalance_change_typesById)
		balance_change_typesGroup.GET("/balance_change_types/typecode/:typecode", h.GetBalance_change_typesByTypeCode)
		balance_change_typesGroup.POST("/balance_change_types", h.InsertBalance_change_types)
		balance_change_typesGroup.PUT("/balance_change_types/:id", h.UpdateBalance_change_types)
		balance_change_typesGroup.DELETE("/balance_change_types/:id", h.DeleteBalance_change_typesById)
	}
}

