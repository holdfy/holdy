package walletsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   walletsSV "palm-pay/app/wallets/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterWalletsHTTPEndpoints(router *echo.Group, uc walletsSV.WalletsServiceIF, log logger.Logger) {
	h := NewWalletsHandler(uc, log)
	walletsGroup := router.Group("/wallets", observabilidade.EnhancedHTTPMetricsMiddleware("wallets"))
	{
		walletsGroup.GET("/wallets", h.GetWallets)
		walletsGroup.GET("/wallets/:id", h.GetWalletsById)
		walletsGroup.GET("/wallets/walletcode/:walletcode", h.GetWalletsByWalletCode)
		walletsGroup.POST("/wallets", h.InsertWallets)
		walletsGroup.PUT("/wallets/:id", h.UpdateWallets)
		walletsGroup.DELETE("/wallets/:id", h.DeleteWalletsById)
	}
}

