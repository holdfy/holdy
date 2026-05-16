package card_brandsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   card_brandsSV "palm-pay/app/card_brands/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterCard_brandsHTTPEndpoints(router *echo.Group, uc card_brandsSV.Card_brandsServiceIF, log logger.Logger) {
	h := NewCard_brandsHandler(uc, log)
	card_brandsGroup := router.Group("/card_brands", observabilidade.EnhancedHTTPMetricsMiddleware("card_brands"))
	{
		card_brandsGroup.GET("/card_brands", h.GetCard_brands)
		card_brandsGroup.GET("/card_brands/:id", h.GetCard_brandsById)
		card_brandsGroup.GET("/card_brands/brandcode/:brandcode", h.GetCard_brandsByBrandCode)
		card_brandsGroup.POST("/card_brands", h.InsertCard_brands)
		card_brandsGroup.PUT("/card_brands/:id", h.UpdateCard_brands)
		card_brandsGroup.DELETE("/card_brands/:id", h.DeleteCard_brandsById)
	}
}

