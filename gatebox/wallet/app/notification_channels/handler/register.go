package notification_channelsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   notification_channelsSV "palm-pay/app/notification_channels/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterNotification_channelsHTTPEndpoints(router *echo.Group, uc notification_channelsSV.Notification_channelsServiceIF, log logger.Logger) {
	h := NewNotification_channelsHandler(uc, log)
	notification_channelsGroup := router.Group("/notification_channels", observabilidade.EnhancedHTTPMetricsMiddleware("notification_channels"))
	{
		notification_channelsGroup.GET("/notification_channels", h.GetNotification_channels)
		notification_channelsGroup.GET("/notification_channels/:id", h.GetNotification_channelsById)
		notification_channelsGroup.GET("/notification_channels/channelcode/:channelcode", h.GetNotification_channelsByChannelCode)
		notification_channelsGroup.POST("/notification_channels", h.InsertNotification_channels)
		notification_channelsGroup.PUT("/notification_channels/:id", h.UpdateNotification_channels)
		notification_channelsGroup.DELETE("/notification_channels/:id", h.DeleteNotification_channelsById)
	}
}

