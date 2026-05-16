package security_event_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  security_event_typesSV "palm-pay/app/security_event_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF security_event_typesSV.Security_event_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewSecurity_event_typesHandler(service security_event_typesSV.Security_event_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("security_event_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetSecurity_event_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_event_types.Handle -> handler.GetSecurity_event_typess.GetSecurity_event_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSecurity_event_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetSecurity_event_typess.Limit", limitParam)
	tracker.AddParam("handler.GetSecurity_event_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetSecurity_event_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Security_event_types); ok {
		tracker.AddResult("handler.GetSecurity_event_typess.Count", len(items))
		tracker.AddResult("handler.GetSecurity_event_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetSecurity_event_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_event_types.Handle -> GetSecurity_event_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSecurity_event_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetSecurity_event_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetSecurity_event_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetSecurity_event_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetSecurity_event_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetSecurity_event_typesByTypeCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_event_typesHandle -> GetSecurity_event_typesByTypeCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSecurity_event_typesByTypeCode")
	defer tracker.Finish(&err)

	typecodeParam := c.Param("typecode")

	typecode := typecodeParam

	result, err := h.serviceIF.GetSecurity_event_typesByTypeCode(c.Request().Context(), typecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetSecurity_event_typesByTypeCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteSecurity_event_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_event_typesHandle -> DeleteSecurity_event_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteSecurity_event_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteSecurity_event_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteSecurity_event_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteSecurity_event_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteSecurity_event_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertSecurity_event_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_event_typesHandle -> InsertSecurity_event_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertSecurity_event_types")
	defer tracker.Finish(&err)




	security_event_types := new(model.Security_event_types)
	errBind := c.Bind(security_event_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertSecurity_event_types(c.Request().Context(), security_event_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertSecurity_event_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateSecurity_event_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_event_typesHandle -> UpdateSecurity_event_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateSecurity_event_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateSecurity_event_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	security_event_types := new(model.Security_event_types)
	errBind := c.Bind(security_event_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateSecurity_event_types(c.Request().Context(), security_event_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateSecurity_event_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

