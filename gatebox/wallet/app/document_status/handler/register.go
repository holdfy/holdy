package document_statusHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   document_statusSV "palm-pay/app/document_status/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterDocument_statusHTTPEndpoints(router *echo.Group, uc document_statusSV.Document_statusServiceIF, log logger.Logger) {
	h := NewDocument_statusHandler(uc, log)
	document_statusGroup := router.Group("/document_status", observabilidade.EnhancedHTTPMetricsMiddleware("document_status"))
	{
		document_statusGroup.GET("/document_status", h.GetDocument_status)
		document_statusGroup.GET("/document_status/:id", h.GetDocument_statusById)
		document_statusGroup.GET("/document_status/statuscode/:statuscode", h.GetDocument_statusByStatusCode)
		document_statusGroup.POST("/document_status", h.InsertDocument_status)
		document_statusGroup.PUT("/document_status/:id", h.UpdateDocument_status)
		document_statusGroup.DELETE("/document_status/:id", h.DeleteDocument_statusById)
	}
}

