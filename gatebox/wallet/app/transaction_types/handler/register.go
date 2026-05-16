package transaction_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   transaction_typesSV "palm-pay/app/transaction_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterTransaction_typesHTTPEndpoints(router *echo.Group, uc transaction_typesSV.Transaction_typesServiceIF, log logger.Logger) {
	h := NewTransaction_typesHandler(uc, log)
	transaction_typesGroup := router.Group("/transaction_types", observabilidade.EnhancedHTTPMetricsMiddleware("transaction_types"))
	{
		transaction_typesGroup.GET("/transaction_types", h.GetTransaction_types)
		transaction_typesGroup.GET("/transaction_types/:id", h.GetTransaction_typesById)
		transaction_typesGroup.GET("/transaction_types/typecode/:typecode", h.GetTransaction_typesByTypeCode)
		transaction_typesGroup.POST("/transaction_types", h.InsertTransaction_types)
		transaction_typesGroup.PUT("/transaction_types/:id", h.UpdateTransaction_types)
		transaction_typesGroup.DELETE("/transaction_types/:id", h.DeleteTransaction_typesById)
	}
}

