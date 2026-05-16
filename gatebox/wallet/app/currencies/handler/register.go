package currenciesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   currenciesSV "palm-pay/app/currencies/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterCurrenciesHTTPEndpoints(router *echo.Group, uc currenciesSV.CurrenciesServiceIF, log logger.Logger) {
	h := NewCurrenciesHandler(uc, log)
	currenciesGroup := router.Group("/currencies", observabilidade.EnhancedHTTPMetricsMiddleware("currencies"))
	{
		currenciesGroup.GET("/currencies", h.GetCurrencies)
		currenciesGroup.GET("/currencies/:id", h.GetCurrenciesById)
		currenciesGroup.GET("/currencies/currencycode/:currencycode", h.GetCurrenciesByCurrencyCode)
		currenciesGroup.POST("/currencies", h.InsertCurrencies)
		currenciesGroup.PUT("/currencies/:id", h.UpdateCurrencies)
		currenciesGroup.DELETE("/currencies/:id", h.DeleteCurrenciesById)
	}
}

