package notification_historyHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   notification_historySV "palm-pay/app/notification_history/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterNotification_historyHTTPEndpoints(router *echo.Group, uc notification_historySV.Notification_historyServiceIF, log logger.Logger) {
	h := NewNotification_historyHandler(uc, log)
	notification_historyGroup := router.Group("/notification_history", observabilidade.EnhancedHTTPMetricsMiddleware("notification_history"))
	{
		notification_historyGroup.GET("/notification_history", h.GetNotification_history)
		notification_historyGroup.GET("/notification_history/:id", h.GetNotification_historyById)
		notification_historyGroup.GET("/notification_history/notificationcode/:notificationcode", h.GetNotification_historyByNotificationCode)
		notification_historyGroup.POST("/notification_history", h.InsertNotification_history)
		notification_historyGroup.PUT("/notification_history/:id", h.UpdateNotification_history)
		notification_historyGroup.DELETE("/notification_history/:id", h.DeleteNotification_historyById)
	}
}

