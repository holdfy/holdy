package document_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   document_typesSV "palm-pay/app/document_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterDocument_typesHTTPEndpoints(router *echo.Group, uc document_typesSV.Document_typesServiceIF, log logger.Logger) {
	h := NewDocument_typesHandler(uc, log)
	document_typesGroup := router.Group("/document_types", observabilidade.EnhancedHTTPMetricsMiddleware("document_types"))
	{
		document_typesGroup.GET("/document_types", h.GetDocument_types)
		document_typesGroup.GET("/document_types/:id", h.GetDocument_typesById)
		document_typesGroup.GET("/document_types/typecode/:typecode", h.GetDocument_typesByTypeCode)
		document_typesGroup.POST("/document_types", h.InsertDocument_types)
		document_typesGroup.PUT("/document_types/:id", h.UpdateDocument_types)
		document_typesGroup.DELETE("/document_types/:id", h.DeleteDocument_typesById)
	}
}

