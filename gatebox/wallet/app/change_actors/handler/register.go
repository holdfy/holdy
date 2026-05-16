package change_actorsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   change_actorsSV "palm-pay/app/change_actors/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterChange_actorsHTTPEndpoints(router *echo.Group, uc change_actorsSV.Change_actorsServiceIF, log logger.Logger) {
	h := NewChange_actorsHandler(uc, log)
	change_actorsGroup := router.Group("/change_actors", observabilidade.EnhancedHTTPMetricsMiddleware("change_actors"))
	{
		change_actorsGroup.GET("/change_actors", h.GetChange_actors)
		change_actorsGroup.GET("/change_actors/:id", h.GetChange_actorsById)
		change_actorsGroup.GET("/change_actors/actorcode/:actorcode", h.GetChange_actorsByActorCode)
		change_actorsGroup.POST("/change_actors", h.InsertChange_actors)
		change_actorsGroup.PUT("/change_actors/:id", h.UpdateChange_actors)
		change_actorsGroup.DELETE("/change_actors/:id", h.DeleteChange_actorsById)
	}
}

