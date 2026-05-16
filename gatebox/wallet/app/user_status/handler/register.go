package user_statusHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   user_statusSV "palm-pay/app/user_status/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterUser_statusHTTPEndpoints(router *echo.Group, uc user_statusSV.User_statusServiceIF, log logger.Logger) {
	h := NewUser_statusHandler(uc, log)
	user_statusGroup := router.Group("/user_status", observabilidade.EnhancedHTTPMetricsMiddleware("user_status"))
	{
		user_statusGroup.GET("/user_status", h.GetUser_status)
		user_statusGroup.GET("/user_status/:id", h.GetUser_statusById)
		user_statusGroup.GET("/user_status/statuscode/:statuscode", h.GetUser_statusByStatusCode)
		user_statusGroup.POST("/user_status", h.InsertUser_status)
		user_statusGroup.PUT("/user_status/:id", h.UpdateUser_status)
		user_statusGroup.DELETE("/user_status/:id", h.DeleteUser_statusById)
	}
}

