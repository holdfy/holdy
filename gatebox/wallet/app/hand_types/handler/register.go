package hand_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   hand_typesSV "palm-pay/app/hand_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterHand_typesHTTPEndpoints(router *echo.Group, uc hand_typesSV.Hand_typesServiceIF, log logger.Logger) {
	h := NewHand_typesHandler(uc, log)
	hand_typesGroup := router.Group("/hand_types", observabilidade.EnhancedHTTPMetricsMiddleware("hand_types"))
	{
		hand_typesGroup.GET("/hand_types", h.GetHand_types)
		hand_typesGroup.GET("/hand_types/:id", h.GetHand_typesById)
		hand_typesGroup.GET("/hand_types/typecode/:typecode", h.GetHand_typesByTypeCode)
		hand_typesGroup.POST("/hand_types", h.InsertHand_types)
		hand_typesGroup.PUT("/hand_types/:id", h.UpdateHand_types)
		hand_typesGroup.DELETE("/hand_types/:id", h.DeleteHand_typesById)
	}
}

