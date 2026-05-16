package security_eventsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  security_eventsSV "palm-pay/app/security_events/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF security_eventsSV.Security_eventsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewSecurity_eventsHandler(service security_eventsSV.Security_eventsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("security_events"), // <---- adicionado aqui
     }
}
func (h Handler)  GetSecurity_events(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_events.Handle -> handler.GetSecurity_eventss.GetSecurity_events", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSecurity_eventss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetSecurity_eventss.Limit", limitParam)
	tracker.AddParam("handler.GetSecurity_eventss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetSecurity_events(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Security_events); ok {
		tracker.AddResult("handler.GetSecurity_eventss.Count", len(items))
		tracker.AddResult("handler.GetSecurity_eventss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetSecurity_eventsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_events.Handle -> GetSecurity_eventsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSecurity_eventsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetSecurity_eventsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetSecurity_eventsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetSecurity_eventsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetSecurity_eventsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetSecurity_eventsBySecurityEventCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_eventsHandle -> GetSecurity_eventsBySecurityEventCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSecurity_eventsBySecurityEventCode")
	defer tracker.Finish(&err)

	securityeventcodeParam := c.Param("securityeventcode")

	securityeventcode := securityeventcodeParam

	result, err := h.serviceIF.GetSecurity_eventsBySecurityEventCode(c.Request().Context(), securityeventcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetSecurity_eventsBySecurityEventCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteSecurity_eventsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_eventsHandle -> DeleteSecurity_eventsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteSecurity_eventsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteSecurity_eventsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteSecurity_eventsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteSecurity_eventsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteSecurity_eventsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertSecurity_events(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_eventsHandle -> InsertSecurity_events", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertSecurity_events")
	defer tracker.Finish(&err)




	security_events := new(model.Security_events)
	errBind := c.Bind(security_events)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertSecurity_events(c.Request().Context(), security_events)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertSecurity_events.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateSecurity_events(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_eventsHandle -> UpdateSecurity_events", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateSecurity_events")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateSecurity_events.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	security_events := new(model.Security_events)
	errBind := c.Bind(security_events)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateSecurity_events(c.Request().Context(), security_events, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateSecurity_events.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

