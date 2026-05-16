package payment_methodsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   payment_methodsSV "palm-pay/app/payment_methods/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterPayment_methodsHTTPEndpoints(router *echo.Group, uc payment_methodsSV.Payment_methodsServiceIF, log logger.Logger) {
	h := NewPayment_methodsHandler(uc, log)
	payment_methodsGroup := router.Group("/payment_methods", observabilidade.EnhancedHTTPMetricsMiddleware("payment_methods"))
	{
		payment_methodsGroup.GET("/payment_methods", h.GetPayment_methods)
		payment_methodsGroup.GET("/payment_methods/:id", h.GetPayment_methodsById)
		payment_methodsGroup.GET("/payment_methods/methodcode/:methodcode", h.GetPayment_methodsByMethodCode)
		payment_methodsGroup.POST("/payment_methods", h.InsertPayment_methods)
		payment_methodsGroup.PUT("/payment_methods/:id", h.UpdatePayment_methods)
		payment_methodsGroup.DELETE("/payment_methods/:id", h.DeletePayment_methodsById)
	}
}

