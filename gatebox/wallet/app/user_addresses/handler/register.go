package user_addressesHandler

import (
   "github.com/labstack/echo/v4"
	"palm-pay/utils/observabilidade"
   user_addressesSV "palm-pay/app/user_addresses/service"
   "github.com/tungstenbyte/utils/logger"
)
func RegisterUser_addressesHTTPEndpoints(router *echo.Group, uc user_addressesSV.User_addressesServiceIF, log logger.Logger) {
	h := NewUser_addressesHandler(uc, log)
	user_addressesGroup := router.Group("/user_addresses", observabilidade.EnhancedHTTPMetricsMiddleware("user_addresses"))
	{
		user_addressesGroup.GET("/user_addresses", h.GetUser_addresses)
		user_addressesGroup.GET("/user_addresses/:id", h.GetUser_addressesById)
		user_addressesGroup.GET("/user_addresses/addresscode/:addresscode", h.GetUser_addressesByAddressCode)
		user_addressesGroup.POST("/user_addresses", h.InsertUser_addresses)
		user_addressesGroup.PUT("/user_addresses/:id", h.UpdateUser_addresses)
		user_addressesGroup.DELETE("/user_addresses/:id", h.DeleteUser_addressesById)
	}
}

