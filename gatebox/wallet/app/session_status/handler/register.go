package session_statusHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   session_statusSV "palm-pay/app/session_status/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterSession_statusHTTPEndpoints(router *echo.Group, uc session_statusSV.Session_statusServiceIF, log logger.Logger) {
	h := NewSession_statusHandler(uc, log)
	session_statusGroup := router.Group("/session_status", observabilidade.EnhancedHTTPMetricsMiddleware("session_status"))
	{
		session_statusGroup.GET("/session_status", h.GetSession_status)
		session_statusGroup.GET("/session_status/:id", h.GetSession_statusById)
		session_statusGroup.GET("/session_status/statuscode/:statuscode", h.GetSession_statusByStatusCode)
		session_statusGroup.POST("/session_status", h.InsertSession_status)
		session_statusGroup.PUT("/session_status/:id", h.UpdateSession_status)
		session_statusGroup.DELETE("/session_status/:id", h.DeleteSession_statusById)
	}
}

