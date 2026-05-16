package banksHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   banksSV "palm-pay/app/banks/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterBanksHTTPEndpoints(router *echo.Group, uc banksSV.BanksServiceIF, log logger.Logger) {
	h := NewBanksHandler(uc, log)
	banksGroup := router.Group("/banks", observabilidade.EnhancedHTTPMetricsMiddleware("banks"))
	{
		banksGroup.GET("/banks", h.GetBanks)
		banksGroup.GET("/banks/:id", h.GetBanksById)
		banksGroup.GET("/banks/bankcodeinternal/:bankcodeinternal", h.GetBanksByBankCodeInternal)
		banksGroup.POST("/banks", h.InsertBanks)
		banksGroup.PUT("/banks/:id", h.UpdateBanks)
		banksGroup.DELETE("/banks/:id", h.DeleteBanksById)
	}
}

