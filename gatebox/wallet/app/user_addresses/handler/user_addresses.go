package user_addressesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  user_addressesSV "palm-pay/app/user_addresses/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF user_addressesSV.User_addressesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewUser_addressesHandler(service user_addressesSV.User_addressesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("user_addresses"), // <---- adicionado aqui
     }
}
func (h Handler)  GetUser_addresses(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("User_addresses.Handle -> handler.GetUser_addressess.GetUser_addresses", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_addressess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetUser_addressess.Limit", limitParam)
	tracker.AddParam("handler.GetUser_addressess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetUser_addresses(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.User_addresses); ok {
		tracker.AddResult("handler.GetUser_addressess.Count", len(items))
		tracker.AddResult("handler.GetUser_addressess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetUser_addressesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_addresses.Handle -> GetUser_addressesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_addressesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetUser_addressesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetUser_addressesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_addressesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetUser_addressesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetUser_addressesByAddressCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_addressesHandle -> GetUser_addressesByAddressCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_addressesByAddressCode")
	defer tracker.Finish(&err)

	addresscodeParam := c.Param("addresscode")

	addresscode := addresscodeParam

	result, err := h.serviceIF.GetUser_addressesByAddressCode(c.Request().Context(), addresscode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_addressesByAddressCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteUser_addressesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_addressesHandle -> DeleteUser_addressesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteUser_addressesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteUser_addressesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteUser_addressesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteUser_addressesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteUser_addressesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertUser_addresses(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_addressesHandle -> InsertUser_addresses", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertUser_addresses")
	defer tracker.Finish(&err)




	user_addresses := new(model.User_addresses)
	errBind := c.Bind(user_addresses)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertUser_addresses(c.Request().Context(), user_addresses)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertUser_addresses.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateUser_addresses(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_addressesHandle -> UpdateUser_addresses", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateUser_addresses")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateUser_addresses.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	user_addresses := new(model.User_addresses)
	errBind := c.Bind(user_addresses)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateUser_addresses(c.Request().Context(), user_addresses, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateUser_addresses.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

