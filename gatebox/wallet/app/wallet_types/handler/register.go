package wallet_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   wallet_typesSV "palm-pay/app/wallet_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterWallet_typesHTTPEndpoints(router *echo.Group, uc wallet_typesSV.Wallet_typesServiceIF, log logger.Logger) {
	h := NewWallet_typesHandler(uc, log)
	wallet_typesGroup := router.Group("/wallet_types", observabilidade.EnhancedHTTPMetricsMiddleware("wallet_types"))
	{
		wallet_typesGroup.GET("/wallet_types", h.GetWallet_types)
		wallet_typesGroup.GET("/wallet_types/:id", h.GetWallet_typesById)
		wallet_typesGroup.GET("/wallet_types/typecode/:typecode", h.GetWallet_typesByTypeCode)
		wallet_typesGroup.POST("/wallet_types", h.InsertWallet_types)
		wallet_typesGroup.PUT("/wallet_types/:id", h.UpdateWallet_types)
		wallet_typesGroup.DELETE("/wallet_types/:id", h.DeleteWallet_typesById)
	}
}

