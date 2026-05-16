package transaction_status_historyHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   transaction_status_historySV "palm-pay/app/transaction_status_history/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterTransaction_status_historyHTTPEndpoints(router *echo.Group, uc transaction_status_historySV.Transaction_status_historyServiceIF, log logger.Logger) {
	h := NewTransaction_status_historyHandler(uc, log)
	transaction_status_historyGroup := router.Group("/transaction_status_history", observabilidade.EnhancedHTTPMetricsMiddleware("transaction_status_history"))
	{
		transaction_status_historyGroup.GET("/transaction_status_history", h.GetTransaction_status_history)
		transaction_status_historyGroup.GET("/transaction_status_history/:id", h.GetTransaction_status_historyById)
		transaction_status_historyGroup.GET("/transaction_status_history/statushistorycode/:statushistorycode", h.GetTransaction_status_historyByStatusHistoryCode)
		transaction_status_historyGroup.POST("/transaction_status_history", h.InsertTransaction_status_history)
		transaction_status_historyGroup.PUT("/transaction_status_history/:id", h.UpdateTransaction_status_history)
		transaction_status_historyGroup.DELETE("/transaction_status_history/:id", h.DeleteTransaction_status_historyById)
	}
}

