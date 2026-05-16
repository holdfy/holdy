package notification_templatesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   notification_templatesSV "palm-pay/app/notification_templates/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterNotification_templatesHTTPEndpoints(router *echo.Group, uc notification_templatesSV.Notification_templatesServiceIF, log logger.Logger) {
	h := NewNotification_templatesHandler(uc, log)
	notification_templatesGroup := router.Group("/notification_templates", observabilidade.EnhancedHTTPMetricsMiddleware("notification_templates"))
	{
		notification_templatesGroup.GET("/notification_templates", h.GetNotification_templates)
		notification_templatesGroup.GET("/notification_templates/:id", h.GetNotification_templatesById)
		notification_templatesGroup.GET("/notification_templates/templatecode/:templatecode", h.GetNotification_templatesByTemplateCode)
		notification_templatesGroup.POST("/notification_templates", h.InsertNotification_templates)
		notification_templatesGroup.PUT("/notification_templates/:id", h.UpdateNotification_templates)
		notification_templatesGroup.DELETE("/notification_templates/:id", h.DeleteNotification_templatesById)
	}
}

