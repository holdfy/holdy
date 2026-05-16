package palm_biometricsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   palm_biometricsSV "palm-pay/app/palm_biometrics/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterPalm_biometricsHTTPEndpoints(router *echo.Group, uc palm_biometricsSV.Palm_biometricsServiceIF, log logger.Logger) {
	h := NewPalm_biometricsHandler(uc, log)
	palm_biometricsGroup := router.Group("/palm_biometrics", observabilidade.EnhancedHTTPMetricsMiddleware("palm_biometrics"))
	{
		palm_biometricsGroup.GET("/palm_biometrics", h.GetPalm_biometrics)
		palm_biometricsGroup.GET("/palm_biometrics/:id", h.GetPalm_biometricsById)
		palm_biometricsGroup.GET("/palm_biometrics/biometriccode/:biometriccode", h.GetPalm_biometricsByBiometricCode)
		palm_biometricsGroup.POST("/palm_biometrics", h.InsertPalm_biometrics)
		palm_biometricsGroup.PUT("/palm_biometrics/:id", h.UpdatePalm_biometrics)
		palm_biometricsGroup.DELETE("/palm_biometrics/:id", h.DeletePalm_biometricsById)
	}
}

