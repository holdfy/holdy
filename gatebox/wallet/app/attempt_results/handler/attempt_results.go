package attempt_resultsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  attempt_resultsSV "palm-pay/app/attempt_results/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF attempt_resultsSV.Attempt_resultsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewAttempt_resultsHandler(service attempt_resultsSV.Attempt_resultsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("attempt_results"), // <---- adicionado aqui
     }
}
func (h Handler)  GetAttempt_results(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Attempt_results.Handle -> handler.GetAttempt_resultss.GetAttempt_results", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAttempt_resultss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetAttempt_resultss.Limit", limitParam)
	tracker.AddParam("handler.GetAttempt_resultss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetAttempt_results(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Attempt_results); ok {
		tracker.AddResult("handler.GetAttempt_resultss.Count", len(items))
		tracker.AddResult("handler.GetAttempt_resultss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetAttempt_resultsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Attempt_results.Handle -> GetAttempt_resultsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAttempt_resultsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetAttempt_resultsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetAttempt_resultsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAttempt_resultsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetAttempt_resultsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetAttempt_resultsByResultCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Attempt_resultsHandle -> GetAttempt_resultsByResultCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAttempt_resultsByResultCode")
	defer tracker.Finish(&err)

	resultcodeParam := c.Param("resultcode")

	resultcode := resultcodeParam

	result, err := h.serviceIF.GetAttempt_resultsByResultCode(c.Request().Context(), resultcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAttempt_resultsByResultCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteAttempt_resultsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Attempt_resultsHandle -> DeleteAttempt_resultsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteAttempt_resultsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteAttempt_resultsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteAttempt_resultsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteAttempt_resultsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteAttempt_resultsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertAttempt_results(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Attempt_resultsHandle -> InsertAttempt_results", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertAttempt_results")
	defer tracker.Finish(&err)




	attempt_results := new(model.Attempt_results)
	errBind := c.Bind(attempt_results)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertAttempt_results(c.Request().Context(), attempt_results)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertAttempt_results.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateAttempt_results(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Attempt_resultsHandle -> UpdateAttempt_results", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateAttempt_results")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateAttempt_results.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	attempt_results := new(model.Attempt_results)
	errBind := c.Bind(attempt_results)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateAttempt_results(c.Request().Context(), attempt_results, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateAttempt_results.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

