package wallet_statusHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   wallet_statusSV "palm-pay/app/wallet_status/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterWallet_statusHTTPEndpoints(router *echo.Group, uc wallet_statusSV.Wallet_statusServiceIF, log logger.Logger) {
	h := NewWallet_statusHandler(uc, log)
	wallet_statusGroup := router.Group("/wallet_status", observabilidade.EnhancedHTTPMetricsMiddleware("wallet_status"))
	{
		wallet_statusGroup.GET("/wallet_status", h.GetWallet_status)
		wallet_statusGroup.GET("/wallet_status/:id", h.GetWallet_statusById)
		wallet_statusGroup.GET("/wallet_status/statuscode/:statuscode", h.GetWallet_statusByStatusCode)
		wallet_statusGroup.POST("/wallet_status", h.InsertWallet_status)
		wallet_statusGroup.PUT("/wallet_status/:id", h.UpdateWallet_status)
		wallet_statusGroup.DELETE("/wallet_status/:id", h.DeleteWallet_statusById)
	}
}

