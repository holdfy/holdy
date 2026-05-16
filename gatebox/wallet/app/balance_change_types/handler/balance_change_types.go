package balance_change_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  balance_change_typesSV "palm-pay/app/balance_change_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF balance_change_typesSV.Balance_change_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewBalance_change_typesHandler(service balance_change_typesSV.Balance_change_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("balance_change_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetBalance_change_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Balance_change_types.Handle -> handler.GetBalance_change_typess.GetBalance_change_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetBalance_change_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetBalance_change_typess.Limit", limitParam)
	tracker.AddParam("handler.GetBalance_change_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetBalance_change_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Balance_change_types); ok {
		tracker.AddResult("handler.GetBalance_change_typess.Count", len(items))
		tracker.AddResult("handler.GetBalance_change_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetBalance_change_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Balance_change_types.Handle -> GetBalance_change_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetBalance_change_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetBalance_change_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetBalance_change_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetBalance_change_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetBalance_change_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetBalance_change_typesByTypeCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Balance_change_typesHandle -> GetBalance_change_typesByTypeCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetBalance_change_typesByTypeCode")
	defer tracker.Finish(&err)

	typecodeParam := c.Param("typecode")

	typecode := typecodeParam

	result, err := h.serviceIF.GetBalance_change_typesByTypeCode(c.Request().Context(), typecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetBalance_change_typesByTypeCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteBalance_change_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Balance_change_typesHandle -> DeleteBalance_change_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteBalance_change_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteBalance_change_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteBalance_change_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteBalance_change_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteBalance_change_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertBalance_change_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Balance_change_typesHandle -> InsertBalance_change_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertBalance_change_types")
	defer tracker.Finish(&err)




	balance_change_types := new(model.Balance_change_types)
	errBind := c.Bind(balance_change_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertBalance_change_types(c.Request().Context(), balance_change_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertBalance_change_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateBalance_change_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Balance_change_typesHandle -> UpdateBalance_change_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateBalance_change_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateBalance_change_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	balance_change_types := new(model.Balance_change_types)
	errBind := c.Bind(balance_change_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateBalance_change_types(c.Request().Context(), balance_change_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateBalance_change_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

