package transactionsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   transactionsSV "palm-pay/app/transactions/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterTransactionsHTTPEndpoints(router *echo.Group, uc transactionsSV.TransactionsServiceIF, log logger.Logger) {
	h := NewTransactionsHandler(uc, log)
	transactionsGroup := router.Group("/transactions", observabilidade.EnhancedHTTPMetricsMiddleware("transactions"))
	{
		transactionsGroup.GET("/transactions", h.GetTransactions)
		transactionsGroup.GET("/transactions/:id", h.GetTransactionsById)
		transactionsGroup.GET("/transactions/transactioncode/:transactioncode", h.GetTransactionsByTransactionCode)
		transactionsGroup.POST("/transactions", h.InsertTransactions)
		transactionsGroup.PUT("/transactions/:id", h.UpdateTransactions)
		transactionsGroup.DELETE("/transactions/:id", h.DeleteTransactionsById)
	}
}

