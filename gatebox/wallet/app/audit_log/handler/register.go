package audit_logHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   audit_logSV "palm-pay/app/audit_log/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterAudit_logHTTPEndpoints(router *echo.Group, uc audit_logSV.Audit_logServiceIF, log logger.Logger) {
	h := NewAudit_logHandler(uc, log)
	audit_logGroup := router.Group("/audit_log", observabilidade.EnhancedHTTPMetricsMiddleware("audit_log"))
	{
		audit_logGroup.GET("/audit_log", h.GetAudit_log)
		audit_logGroup.GET("/audit_log/:id", h.GetAudit_logById)
		audit_logGroup.GET("/audit_log/auditcode/:auditcode", h.GetAudit_logByAuditCode)
		audit_logGroup.POST("/audit_log", h.InsertAudit_log)
		audit_logGroup.PUT("/audit_log/:id", h.UpdateAudit_log)
		audit_logGroup.DELETE("/audit_log/:id", h.DeleteAudit_logById)
	}
}

