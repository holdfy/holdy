package failure_reasonsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  failure_reasonsSV "palm-pay/app/failure_reasons/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF failure_reasonsSV.Failure_reasonsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewFailure_reasonsHandler(service failure_reasonsSV.Failure_reasonsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("failure_reasons"), // <---- adicionado aqui
     }
}
func (h Handler)  GetFailure_reasons(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Failure_reasons.Handle -> handler.GetFailure_reasonss.GetFailure_reasons", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetFailure_reasonss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetFailure_reasonss.Limit", limitParam)
	tracker.AddParam("handler.GetFailure_reasonss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetFailure_reasons(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Failure_reasons); ok {
		tracker.AddResult("handler.GetFailure_reasonss.Count", len(items))
		tracker.AddResult("handler.GetFailure_reasonss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetFailure_reasonsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Failure_reasons.Handle -> GetFailure_reasonsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetFailure_reasonsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetFailure_reasonsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetFailure_reasonsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetFailure_reasonsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetFailure_reasonsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetFailure_reasonsByReasonCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Failure_reasonsHandle -> GetFailure_reasonsByReasonCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetFailure_reasonsByReasonCode")
	defer tracker.Finish(&err)

	reasoncodeParam := c.Param("reasoncode")

	reasoncode := reasoncodeParam

	result, err := h.serviceIF.GetFailure_reasonsByReasonCode(c.Request().Context(), reasoncode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetFailure_reasonsByReasonCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteFailure_reasonsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Failure_reasonsHandle -> DeleteFailure_reasonsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteFailure_reasonsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteFailure_reasonsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteFailure_reasonsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteFailure_reasonsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteFailure_reasonsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertFailure_reasons(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Failure_reasonsHandle -> InsertFailure_reasons", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertFailure_reasons")
	defer tracker.Finish(&err)




	failure_reasons := new(model.Failure_reasons)
	errBind := c.Bind(failure_reasons)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertFailure_reasons(c.Request().Context(), failure_reasons)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertFailure_reasons.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateFailure_reasons(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Failure_reasonsHandle -> UpdateFailure_reasons", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateFailure_reasons")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateFailure_reasons.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	failure_reasons := new(model.Failure_reasons)
	errBind := c.Bind(failure_reasons)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateFailure_reasons(c.Request().Context(), failure_reasons, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateFailure_reasons.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

