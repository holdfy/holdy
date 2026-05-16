package applicationsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   applicationsSV "palm-pay/app/applications/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterApplicationsHTTPEndpoints(router *echo.Group, uc applicationsSV.ApplicationsServiceIF, log logger.Logger) {
	h := NewApplicationsHandler(uc, log)
	applicationsGroup := router.Group("/applications", observabilidade.EnhancedHTTPMetricsMiddleware("applications"))
	{
		applicationsGroup.GET("/applications", h.GetApplications)
		applicationsGroup.GET("/applications/:id", h.GetApplicationsById)
		applicationsGroup.GET("/applications/appcode/:appcode", h.GetApplicationsByAppCode)
		applicationsGroup.POST("/applications", h.InsertApplications)
		applicationsGroup.PUT("/applications/:id", h.UpdateApplications)
		applicationsGroup.DELETE("/applications/:id", h.DeleteApplicationsById)
	}
}

