package signature_methodsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   signature_methodsSV "palm-pay/app/signature_methods/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterSignature_methodsHTTPEndpoints(router *echo.Group, uc signature_methodsSV.Signature_methodsServiceIF, log logger.Logger) {
	h := NewSignature_methodsHandler(uc, log)
	signature_methodsGroup := router.Group("/signature_methods", observabilidade.EnhancedHTTPMetricsMiddleware("signature_methods"))
	{
		signature_methodsGroup.GET("/signature_methods", h.GetSignature_methods)
		signature_methodsGroup.GET("/signature_methods/:id", h.GetSignature_methodsById)
		signature_methodsGroup.GET("/signature_methods/methodcode/:methodcode", h.GetSignature_methodsByMethodCode)
		signature_methodsGroup.POST("/signature_methods", h.InsertSignature_methods)
		signature_methodsGroup.PUT("/signature_methods/:id", h.UpdateSignature_methods)
		signature_methodsGroup.DELETE("/signature_methods/:id", h.DeleteSignature_methodsById)
	}
}

