package security_severity_levelsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   security_severity_levelsSV "palm-pay/app/security_severity_levels/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterSecurity_severity_levelsHTTPEndpoints(router *echo.Group, uc security_severity_levelsSV.Security_severity_levelsServiceIF, log logger.Logger) {
	h := NewSecurity_severity_levelsHandler(uc, log)
	security_severity_levelsGroup := router.Group("/security_severity_levels", observabilidade.EnhancedHTTPMetricsMiddleware("security_severity_levels"))
	{
		security_severity_levelsGroup.GET("/security_severity_levels", h.GetSecurity_severity_levels)
		security_severity_levelsGroup.GET("/security_severity_levels/:id", h.GetSecurity_severity_levelsById)
		security_severity_levelsGroup.GET("/security_severity_levels/severitycode/:severitycode", h.GetSecurity_severity_levelsBySeverityCode)
		security_severity_levelsGroup.POST("/security_severity_levels", h.InsertSecurity_severity_levels)
		security_severity_levelsGroup.PUT("/security_severity_levels/:id", h.UpdateSecurity_severity_levels)
		security_severity_levelsGroup.DELETE("/security_severity_levels/:id", h.DeleteSecurity_severity_levelsById)
	}
}

