package user_cardsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   user_cardsSV "palm-pay/app/user_cards/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterUser_cardsHTTPEndpoints(router *echo.Group, uc user_cardsSV.User_cardsServiceIF, log logger.Logger) {
	h := NewUser_cardsHandler(uc, log)
	user_cardsGroup := router.Group("/user_cards", observabilidade.EnhancedHTTPMetricsMiddleware("user_cards"))
	{
		user_cardsGroup.GET("/user_cards", h.GetUser_cards)
		user_cardsGroup.GET("/user_cards/:id", h.GetUser_cardsById)
		user_cardsGroup.GET("/user_cards/cardcode/:cardcode", h.GetUser_cardsByCardCode)
		user_cardsGroup.POST("/user_cards", h.InsertUser_cards)
		user_cardsGroup.PUT("/user_cards/:id", h.UpdateUser_cards)
		user_cardsGroup.DELETE("/user_cards/:id", h.DeleteUser_cardsById)
	}
}

