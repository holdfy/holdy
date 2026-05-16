package user_sessionsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   user_sessionsSV "palm-pay/app/user_sessions/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterUser_sessionsHTTPEndpoints(router *echo.Group, uc user_sessionsSV.User_sessionsServiceIF, log logger.Logger) {
	h := NewUser_sessionsHandler(uc, log)
	user_sessionsGroup := router.Group("/user_sessions", observabilidade.EnhancedHTTPMetricsMiddleware("user_sessions"))
	{
		user_sessionsGroup.GET("/user_sessions", h.GetUser_sessions)
		user_sessionsGroup.GET("/user_sessions/:id", h.GetUser_sessionsById)
		user_sessionsGroup.GET("/user_sessions/sessioncode/:sessioncode", h.GetUser_sessionsBySessionCode)
		user_sessionsGroup.POST("/user_sessions", h.InsertUser_sessions)
		user_sessionsGroup.PUT("/user_sessions/:id", h.UpdateUser_sessions)
		user_sessionsGroup.DELETE("/user_sessions/:id", h.DeleteUser_sessionsById)
	}
}

