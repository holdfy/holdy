package external_walletsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   external_walletsSV "palm-pay/app/external_wallets/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterExternal_walletsHTTPEndpoints(router *echo.Group, uc external_walletsSV.External_walletsServiceIF, log logger.Logger) {
	h := NewExternal_walletsHandler(uc, log)
	external_walletsGroup := router.Group("/external_wallets", observabilidade.EnhancedHTTPMetricsMiddleware("external_wallets"))
	{
		external_walletsGroup.GET("/external_wallets", h.GetExternal_wallets)
		external_walletsGroup.GET("/external_wallets/:id", h.GetExternal_walletsById)
		external_walletsGroup.GET("/external_wallets/externalwalletcode/:externalwalletcode", h.GetExternal_walletsByExternalWalletCode)
		external_walletsGroup.POST("/external_wallets", h.InsertExternal_wallets)
		external_walletsGroup.PUT("/external_wallets/:id", h.UpdateExternal_wallets)
		external_walletsGroup.DELETE("/external_wallets/:id", h.DeleteExternal_walletsById)
	}
}

