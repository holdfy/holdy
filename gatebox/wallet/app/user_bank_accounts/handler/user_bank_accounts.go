package user_bank_accountsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  user_bank_accountsSV "palm-pay/app/user_bank_accounts/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF user_bank_accountsSV.User_bank_accountsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewUser_bank_accountsHandler(service user_bank_accountsSV.User_bank_accountsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("user_bank_accounts"), // <---- adicionado aqui
     }
}
func (h Handler)  GetUser_bank_accounts(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("User_bank_accounts.Handle -> handler.GetUser_bank_accountss.GetUser_bank_accounts", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_bank_accountss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetUser_bank_accountss.Limit", limitParam)
	tracker.AddParam("handler.GetUser_bank_accountss.Offset", offsetParam)
	limit, errLimit := strconv.ParseInt(limitParam, 10, 64)
	if errLimit != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgOffsetLimit,
		})
		return nil
	}

	offset, errOffset := strconv.ParseInt(offsetParam, 10, 64)
	if errOffset != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgOffsetLimit,
		})
		return nil
	}

	result, err := h.serviceIF.GetUser_bank_accounts(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.User_bank_accounts); ok {
		tracker.AddResult("handler.GetUser_bank_accountss.Count", len(items))
		tracker.AddResult("handler.GetUser_bank_accountss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetUser_bank_accountsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_bank_accounts.Handle -> GetUser_bank_accountsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_bank_accountsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetUser_bank_accountsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetUser_bank_accountsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_bank_accountsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetUser_bank_accountsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetUser_bank_accountsByBankAccountCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_bank_accountsHandle -> GetUser_bank_accountsByBankAccountCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_bank_accountsByBankAccountCode")
	defer tracker.Finish(&err)

	bankaccountcodeParam := c.Param("bankaccountcode")

	bankaccountcode := bankaccountcodeParam

	result, err := h.serviceIF.GetUser_bank_accountsByBankAccountCode(c.Request().Context(), bankaccountcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_bank_accountsByBankAccountCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteUser_bank_accountsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_bank_accountsHandle -> DeleteUser_bank_accountsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteUser_bank_accountsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteUser_bank_accountsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteUser_bank_accountsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteUser_bank_accountsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteUser_bank_accountsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertUser_bank_accounts(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_bank_accountsHandle -> InsertUser_bank_accounts", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertUser_bank_accounts")
	defer tracker.Finish(&err)




	user_bank_accounts := new(model.User_bank_accounts)
	errBind := c.Bind(user_bank_accounts)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertUser_bank_accounts(c.Request().Context(), user_bank_accounts)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertUser_bank_accounts.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateUser_bank_accounts(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_bank_accountsHandle -> UpdateUser_bank_accounts", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateUser_bank_accounts")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateUser_bank_accounts.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	user_bank_accounts := new(model.User_bank_accounts)
	errBind := c.Bind(user_bank_accounts)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateUser_bank_accounts(c.Request().Context(), user_bank_accounts, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateUser_bank_accounts.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

