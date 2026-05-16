package account_typesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   account_typesSV "palm-pay/app/account_types/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterAccount_typesHTTPEndpoints(router *echo.Group, uc account_typesSV.Account_typesServiceIF, log logger.Logger) {
	h := NewAccount_typesHandler(uc, log)
	account_typesGroup := router.Group("/account_types", observabilidade.EnhancedHTTPMetricsMiddleware("account_types"))
	{
		account_typesGroup.GET("/account_types", h.GetAccount_types)
		account_typesGroup.GET("/account_types/:id", h.GetAccount_typesById)
		account_typesGroup.GET("/account_types/typecode/:typecode", h.GetAccount_typesByTypeCode)
		account_typesGroup.POST("/account_types", h.InsertAccount_types)
		account_typesGroup.PUT("/account_types/:id", h.UpdateAccount_types)
		account_typesGroup.DELETE("/account_types/:id", h.DeleteAccount_typesById)
	}
}

