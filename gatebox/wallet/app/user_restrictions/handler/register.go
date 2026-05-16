package user_restrictionsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   user_restrictionsSV "palm-pay/app/user_restrictions/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterUser_restrictionsHTTPEndpoints(router *echo.Group, uc user_restrictionsSV.User_restrictionsServiceIF, log logger.Logger) {
	h := NewUser_restrictionsHandler(uc, log)
	user_restrictionsGroup := router.Group("/user_restrictions", observabilidade.EnhancedHTTPMetricsMiddleware("user_restrictions"))
	{
		user_restrictionsGroup.GET("/user_restrictions", h.GetUser_restrictions)
		user_restrictionsGroup.GET("/user_restrictions/:id", h.GetUser_restrictionsById)
		user_restrictionsGroup.GET("/user_restrictions/restrictioncode/:restrictioncode", h.GetUser_restrictionsByRestrictionCode)
		user_restrictionsGroup.POST("/user_restrictions", h.InsertUser_restrictions)
		user_restrictionsGroup.PUT("/user_restrictions/:id", h.UpdateUser_restrictions)
		user_restrictionsGroup.DELETE("/user_restrictions/:id", h.DeleteUser_restrictionsById)
	}
}

