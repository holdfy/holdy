package audit_actionsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   audit_actionsSV "palm-pay/app/audit_actions/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterAudit_actionsHTTPEndpoints(router *echo.Group, uc audit_actionsSV.Audit_actionsServiceIF, log logger.Logger) {
	h := NewAudit_actionsHandler(uc, log)
	audit_actionsGroup := router.Group("/audit_actions", observabilidade.EnhancedHTTPMetricsMiddleware("audit_actions"))
	{
		audit_actionsGroup.GET("/audit_actions", h.GetAudit_actions)
		audit_actionsGroup.GET("/audit_actions/:id", h.GetAudit_actionsById)
		audit_actionsGroup.GET("/audit_actions/actioncode/:actioncode", h.GetAudit_actionsByActionCode)
		audit_actionsGroup.POST("/audit_actions", h.InsertAudit_actions)
		audit_actionsGroup.PUT("/audit_actions/:id", h.UpdateAudit_actions)
		audit_actionsGroup.DELETE("/audit_actions/:id", h.DeleteAudit_actionsById)
	}
}

