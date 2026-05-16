package system_configurationsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   system_configurationsSV "palm-pay/app/system_configurations/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterSystem_configurationsHTTPEndpoints(router *echo.Group, uc system_configurationsSV.System_configurationsServiceIF, log logger.Logger) {
	h := NewSystem_configurationsHandler(uc, log)
	system_configurationsGroup := router.Group("/system_configurations", observabilidade.EnhancedHTTPMetricsMiddleware("system_configurations"))
	{
		system_configurationsGroup.GET("/system_configurations", h.GetSystem_configurations)
		system_configurationsGroup.GET("/system_configurations/:id", h.GetSystem_configurationsById)
		system_configurationsGroup.GET("/system_configurations/configcode/:configcode", h.GetSystem_configurationsByConfigCode)
		system_configurationsGroup.POST("/system_configurations", h.InsertSystem_configurations)
		system_configurationsGroup.PUT("/system_configurations/:id", h.UpdateSystem_configurations)
		system_configurationsGroup.DELETE("/system_configurations/:id", h.DeleteSystem_configurationsById)
	}
}

