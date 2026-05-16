package user_bank_accountsHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   user_bank_accountsSV "palm-pay/app/user_bank_accounts/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterUser_bank_accountsHTTPEndpoints(router *echo.Group, uc user_bank_accountsSV.User_bank_accountsServiceIF, log logger.Logger) {
	h := NewUser_bank_accountsHandler(uc, log)
	user_bank_accountsGroup := router.Group("/user_bank_accounts", observabilidade.EnhancedHTTPMetricsMiddleware("user_bank_accounts"))
	{
		user_bank_accountsGroup.GET("/user_bank_accounts", h.GetUser_bank_accounts)
		user_bank_accountsGroup.GET("/user_bank_accounts/:id", h.GetUser_bank_accountsById)
		user_bank_accountsGroup.GET("/user_bank_accounts/bankaccountcode/:bankaccountcode", h.GetUser_bank_accountsByBankAccountCode)
		user_bank_accountsGroup.POST("/user_bank_accounts", h.InsertUser_bank_accounts)
		user_bank_accountsGroup.PUT("/user_bank_accounts/:id", h.UpdateUser_bank_accounts)
		user_bank_accountsGroup.DELETE("/user_bank_accounts/:id", h.DeleteUser_bank_accountsById)
	}
}

