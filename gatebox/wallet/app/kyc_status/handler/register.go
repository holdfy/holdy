package kyc_statusHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   kyc_statusSV "palm-pay/app/kyc_status/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterKyc_statusHTTPEndpoints(router *echo.Group, uc kyc_statusSV.Kyc_statusServiceIF, log logger.Logger) {
	h := NewKyc_statusHandler(uc, log)
	kyc_statusGroup := router.Group("/kyc_status", observabilidade.EnhancedHTTPMetricsMiddleware("kyc_status"))
	{
		kyc_statusGroup.GET("/kyc_status", h.GetKyc_status)
		kyc_statusGroup.GET("/kyc_status/:id", h.GetKyc_statusById)
		kyc_statusGroup.GET("/kyc_status/statuscode/:statuscode", h.GetKyc_statusByStatusCode)
		kyc_statusGroup.POST("/kyc_status", h.InsertKyc_status)
		kyc_statusGroup.PUT("/kyc_status/:id", h.UpdateKyc_status)
		kyc_statusGroup.DELETE("/kyc_status/:id", h.DeleteKyc_statusById)
	}
}

