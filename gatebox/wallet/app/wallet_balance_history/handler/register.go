package wallet_balance_historyHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   wallet_balance_historySV "palm-pay/app/wallet_balance_history/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterWallet_balance_historyHTTPEndpoints(router *echo.Group, uc wallet_balance_historySV.Wallet_balance_historyServiceIF, log logger.Logger) {
	h := NewWallet_balance_historyHandler(uc, log)
	wallet_balance_historyGroup := router.Group("/wallet_balance_history", observabilidade.EnhancedHTTPMetricsMiddleware("wallet_balance_history"))
	{
		wallet_balance_historyGroup.GET("/wallet_balance_history", h.GetWallet_balance_history)
		wallet_balance_historyGroup.GET("/wallet_balance_history/:id", h.GetWallet_balance_historyById)
		wallet_balance_historyGroup.GET("/wallet_balance_history/balancehistorycode/:balancehistorycode", h.GetWallet_balance_historyByBalanceHistoryCode)
		wallet_balance_historyGroup.POST("/wallet_balance_history", h.InsertWallet_balance_history)
		wallet_balance_historyGroup.PUT("/wallet_balance_history/:id", h.UpdateWallet_balance_history)
		wallet_balance_historyGroup.DELETE("/wallet_balance_history/:id", h.DeleteWallet_balance_historyById)
	}
}

