package notification_statusHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   notification_statusSV "palm-pay/app/notification_status/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterNotification_statusHTTPEndpoints(router *echo.Group, uc notification_statusSV.Notification_statusServiceIF, log logger.Logger) {
	h := NewNotification_statusHandler(uc, log)
	notification_statusGroup := router.Group("/notification_status", observabilidade.EnhancedHTTPMetricsMiddleware("notification_status"))
	{
		notification_statusGroup.GET("/notification_status", h.GetNotification_status)
		notification_statusGroup.GET("/notification_status/:id", h.GetNotification_statusById)
		notification_statusGroup.GET("/notification_status/statuscode/:statuscode", h.GetNotification_statusByStatusCode)
		notification_statusGroup.POST("/notification_status", h.InsertNotification_status)
		notification_statusGroup.PUT("/notification_status/:id", h.UpdateNotification_status)
		notification_statusGroup.DELETE("/notification_status/:id", h.DeleteNotification_statusById)
	}
}

