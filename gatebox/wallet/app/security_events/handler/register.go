package security_eventsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   security_eventsSV "palm-pay/app/security_events/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterSecurity_eventsHTTPEndpoints(router *echo.Group, uc security_eventsSV.Security_eventsServiceIF, log logger.Logger) {
	h := NewSecurity_eventsHandler(uc, log)
	security_eventsGroup := router.Group("/security_events", observabilidade.EnhancedHTTPMetricsMiddleware("security_events"))
	{
		security_eventsGroup.GET("/security_events", h.GetSecurity_events)
		security_eventsGroup.GET("/security_events/:id", h.GetSecurity_eventsById)
		security_eventsGroup.GET("/security_events/securityeventcode/:securityeventcode", h.GetSecurity_eventsBySecurityEventCode)
		security_eventsGroup.POST("/security_events", h.InsertSecurity_events)
		security_eventsGroup.PUT("/security_events/:id", h.UpdateSecurity_events)
		security_eventsGroup.DELETE("/security_events/:id", h.DeleteSecurity_eventsById)
	}
}

