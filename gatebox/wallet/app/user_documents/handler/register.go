package user_documentsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   user_documentsSV "palm-pay/app/user_documents/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterUser_documentsHTTPEndpoints(router *echo.Group, uc user_documentsSV.User_documentsServiceIF, log logger.Logger) {
	h := NewUser_documentsHandler(uc, log)
	user_documentsGroup := router.Group("/user_documents", observabilidade.EnhancedHTTPMetricsMiddleware("user_documents"))
	{
		user_documentsGroup.GET("/user_documents", h.GetUser_documents)
		user_documentsGroup.GET("/user_documents/:id", h.GetUser_documentsById)
		user_documentsGroup.GET("/user_documents/documentcode/:documentcode", h.GetUser_documentsByDocumentCode)
		user_documentsGroup.POST("/user_documents", h.InsertUser_documents)
		user_documentsGroup.PUT("/user_documents/:id", h.UpdateUser_documents)
		user_documentsGroup.DELETE("/user_documents/:id", h.DeleteUser_documentsById)
	}
}

