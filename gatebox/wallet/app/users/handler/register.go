package usersHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   usersSV "palm-pay/app/users/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterUsersHTTPEndpoints(router *echo.Group, uc usersSV.UsersServiceIF, log logger.Logger) {
	h := NewUsersHandler(uc, log)
	usersGroup := router.Group("/users", observabilidade.EnhancedHTTPMetricsMiddleware("users"))
	{
		usersGroup.GET("/users", h.GetUsers)
		usersGroup.GET("/users/:id", h.GetUsersById)
		usersGroup.GET("/users/usercode/:usercode", h.GetUsersByUserCode)
		usersGroup.POST("/users", h.InsertUsers)
		usersGroup.PUT("/users/:id", h.UpdateUsers)
		usersGroup.DELETE("/users/:id", h.DeleteUsersById)
	}
}

