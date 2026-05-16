package gateway_transactionsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   gateway_transactionsSV "palm-pay/app/gateway_transactions/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterGateway_transactionsHTTPEndpoints(router *echo.Group, uc gateway_transactionsSV.Gateway_transactionsServiceIF, log logger.Logger) {
	h := NewGateway_transactionsHandler(uc, log)
	gateway_transactionsGroup := router.Group("/gateway_transactions", observabilidade.EnhancedHTTPMetricsMiddleware("gateway_transactions"))
	{
		gateway_transactionsGroup.GET("/gateway_transactions", h.GetGateway_transactions)
		gateway_transactionsGroup.GET("/gateway_transactions/:id", h.GetGateway_transactionsById)
		gateway_transactionsGroup.GET("/gateway_transactions/gatewaytransactioncode/:gatewaytransactioncode", h.GetGateway_transactionsByGatewayTransactionCode)
		gateway_transactionsGroup.POST("/gateway_transactions", h.InsertGateway_transactions)
		gateway_transactionsGroup.PUT("/gateway_transactions/:id", h.UpdateGateway_transactions)
		gateway_transactionsGroup.DELETE("/gateway_transactions/:id", h.DeleteGateway_transactionsById)
	}
}

