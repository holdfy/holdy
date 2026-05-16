package wallet_providersHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   wallet_providersSV "palm-pay/app/wallet_providers/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterWallet_providersHTTPEndpoints(router *echo.Group, uc wallet_providersSV.Wallet_providersServiceIF, log logger.Logger) {
	h := NewWallet_providersHandler(uc, log)
	wallet_providersGroup := router.Group("/wallet_providers", observabilidade.EnhancedHTTPMetricsMiddleware("wallet_providers"))
	{
		wallet_providersGroup.GET("/wallet_providers", h.GetWallet_providers)
		wallet_providersGroup.GET("/wallet_providers/:id", h.GetWallet_providersById)
		wallet_providersGroup.GET("/wallet_providers/providercode/:providercode", h.GetWallet_providersByProviderCode)
		wallet_providersGroup.POST("/wallet_providers", h.InsertWallet_providers)
		wallet_providersGroup.PUT("/wallet_providers/:id", h.UpdateWallet_providers)
		wallet_providersGroup.DELETE("/wallet_providers/:id", h.DeleteWallet_providersById)
	}
}

