package transaction_statusHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  transaction_statusSV "palm-pay/app/transaction_status/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF transaction_statusSV.Transaction_statusServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewTransaction_statusHandler(service transaction_statusSV.Transaction_statusServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("transaction_status"), // <---- adicionado aqui
     }
}
func (h Handler)  GetTransaction_status(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_status.Handle -> handler.GetTransaction_statuss.GetTransaction_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransaction_statuss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetTransaction_statuss.Limit", limitParam)
	tracker.AddParam("handler.GetTransaction_statuss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetTransaction_status(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Transaction_status); ok {
		tracker.AddResult("handler.GetTransaction_statuss.Count", len(items))
		tracker.AddResult("handler.GetTransaction_statuss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetTransaction_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_status.Handle -> GetTransaction_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransaction_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetTransaction_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetTransaction_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetTransaction_statusById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetTransaction_statusById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetTransaction_statusByStatusCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_statusHandle -> GetTransaction_statusByStatusCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransaction_statusByStatusCode")
	defer tracker.Finish(&err)

	statuscodeParam := c.Param("statuscode")

	statuscode := statuscodeParam

	result, err := h.serviceIF.GetTransaction_statusByStatusCode(c.Request().Context(), statuscode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetTransaction_statusByStatusCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteTransaction_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_statusHandle -> DeleteTransaction_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteTransaction_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteTransaction_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteTransaction_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteTransaction_statusById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteTransaction_statusById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertTransaction_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_statusHandle -> InsertTransaction_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertTransaction_status")
	defer tracker.Finish(&err)




	transaction_status := new(model.Transaction_status)
	errBind := c.Bind(transaction_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertTransaction_status(c.Request().Context(), transaction_status)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertTransaction_status.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateTransaction_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_statusHandle -> UpdateTransaction_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateTransaction_status")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateTransaction_status.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	transaction_status := new(model.Transaction_status)
	errBind := c.Bind(transaction_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateTransaction_status(c.Request().Context(), transaction_status, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateTransaction_status.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

