package transaction_statusHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   transaction_statusSV "palm-pay/app/transaction_status/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterTransaction_statusHTTPEndpoints(router *echo.Group, uc transaction_statusSV.Transaction_statusServiceIF, log logger.Logger) {
	h := NewTransaction_statusHandler(uc, log)
	transaction_statusGroup := router.Group("/transaction_status", observabilidade.EnhancedHTTPMetricsMiddleware("transaction_status"))
	{
		transaction_statusGroup.GET("/transaction_status", h.GetTransaction_status)
		transaction_statusGroup.GET("/transaction_status/:id", h.GetTransaction_statusById)
		transaction_statusGroup.GET("/transaction_status/statuscode/:statuscode", h.GetTransaction_statusByStatusCode)
		transaction_statusGroup.POST("/transaction_status", h.InsertTransaction_status)
		transaction_statusGroup.PUT("/transaction_status/:id", h.UpdateTransaction_status)
		transaction_statusGroup.DELETE("/transaction_status/:id", h.DeleteTransaction_statusById)
	}
}

