package account_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  account_typesSV "palm-pay/app/account_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF account_typesSV.Account_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewAccount_typesHandler(service account_typesSV.Account_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("account_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetAccount_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Account_types.Handle -> handler.GetAccount_typess.GetAccount_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAccount_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetAccount_typess.Limit", limitParam)
	tracker.AddParam("handler.GetAccount_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetAccount_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Account_types); ok {
		tracker.AddResult("handler.GetAccount_typess.Count", len(items))
		tracker.AddResult("handler.GetAccount_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetAccount_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Account_types.Handle -> GetAccount_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAccount_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetAccount_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetAccount_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAccount_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetAccount_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetAccount_typesByTypeCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Account_typesHandle -> GetAccount_typesByTypeCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAccount_typesByTypeCode")
	defer tracker.Finish(&err)

	typecodeParam := c.Param("typecode")

	typecode := typecodeParam

	result, err := h.serviceIF.GetAccount_typesByTypeCode(c.Request().Context(), typecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAccount_typesByTypeCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteAccount_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Account_typesHandle -> DeleteAccount_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteAccount_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteAccount_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteAccount_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteAccount_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteAccount_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertAccount_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Account_typesHandle -> InsertAccount_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertAccount_types")
	defer tracker.Finish(&err)




	account_types := new(model.Account_types)
	errBind := c.Bind(account_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertAccount_types(c.Request().Context(), account_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertAccount_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateAccount_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Account_typesHandle -> UpdateAccount_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateAccount_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateAccount_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	account_types := new(model.Account_types)
	errBind := c.Bind(account_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateAccount_types(c.Request().Context(), account_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateAccount_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

