package audit_tablesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   audit_tablesSV "palm-pay/app/audit_tables/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterAudit_tablesHTTPEndpoints(router *echo.Group, uc audit_tablesSV.Audit_tablesServiceIF, log logger.Logger) {
	h := NewAudit_tablesHandler(uc, log)
	audit_tablesGroup := router.Group("/audit_tables", observabilidade.EnhancedHTTPMetricsMiddleware("audit_tables"))
	{
		audit_tablesGroup.GET("/audit_tables", h.GetAudit_tables)
		audit_tablesGroup.GET("/audit_tables/:id", h.GetAudit_tablesById)
		audit_tablesGroup.GET("/audit_tables/tablecode/:tablecode", h.GetAudit_tablesByTableCode)
		audit_tablesGroup.POST("/audit_tables", h.InsertAudit_tables)
		audit_tablesGroup.PUT("/audit_tables/:id", h.UpdateAudit_tables)
		audit_tablesGroup.DELETE("/audit_tables/:id", h.DeleteAudit_tablesById)
	}
}

