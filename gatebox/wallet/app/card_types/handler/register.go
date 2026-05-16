package card_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   card_typesSV "palm-pay/app/card_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterCard_typesHTTPEndpoints(router *echo.Group, uc card_typesSV.Card_typesServiceIF, log logger.Logger) {
	h := NewCard_typesHandler(uc, log)
	card_typesGroup := router.Group("/card_types", observabilidade.EnhancedHTTPMetricsMiddleware("card_types"))
	{
		card_typesGroup.GET("/card_types", h.GetCard_types)
		card_typesGroup.GET("/card_types/:id", h.GetCard_typesById)
		card_typesGroup.GET("/card_types/typecode/:typecode", h.GetCard_typesByTypeCode)
		card_typesGroup.POST("/card_types", h.InsertCard_types)
		card_typesGroup.PUT("/card_types/:id", h.UpdateCard_types)
		card_typesGroup.DELETE("/card_types/:id", h.DeleteCard_typesById)
	}
}

