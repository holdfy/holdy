package transaction_status_historyHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  transaction_status_historySV "palm-pay/app/transaction_status_history/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF transaction_status_historySV.Transaction_status_historyServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewTransaction_status_historyHandler(service transaction_status_historySV.Transaction_status_historyServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("transaction_status_history"), // <---- adicionado aqui
     }
}
func (h Handler)  GetTransaction_status_history(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_status_history.Handle -> handler.GetTransaction_status_historys.GetTransaction_status_history", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransaction_status_historys")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetTransaction_status_historys.Limit", limitParam)
	tracker.AddParam("handler.GetTransaction_status_historys.Offset", offsetParam)
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

	result, err := h.serviceIF.GetTransaction_status_history(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Transaction_status_history); ok {
		tracker.AddResult("handler.GetTransaction_status_historys.Count", len(items))
		tracker.AddResult("handler.GetTransaction_status_historys.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetTransaction_status_historyById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_status_history.Handle -> GetTransaction_status_historyById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransaction_status_historyById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetTransaction_status_historyById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetTransaction_status_historyById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetTransaction_status_historyById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetTransaction_status_historyById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetTransaction_status_historyByStatusHistoryCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_status_historyHandle -> GetTransaction_status_historyByStatusHistoryCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransaction_status_historyByStatusHistoryCode")
	defer tracker.Finish(&err)

	statushistorycodeParam := c.Param("statushistorycode")

	statushistorycode := statushistorycodeParam

	result, err := h.serviceIF.GetTransaction_status_historyByStatusHistoryCode(c.Request().Context(), statushistorycode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetTransaction_status_historyByStatusHistoryCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteTransaction_status_historyById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_status_historyHandle -> DeleteTransaction_status_historyById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteTransaction_status_historyById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteTransaction_status_historyById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteTransaction_status_historyById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteTransaction_status_historyById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteTransaction_status_historyById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertTransaction_status_history(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_status_historyHandle -> InsertTransaction_status_history", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertTransaction_status_history")
	defer tracker.Finish(&err)




	transaction_status_history := new(model.Transaction_status_history)
	errBind := c.Bind(transaction_status_history)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertTransaction_status_history(c.Request().Context(), transaction_status_history)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertTransaction_status_history.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateTransaction_status_history(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_status_historyHandle -> UpdateTransaction_status_history", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateTransaction_status_history")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateTransaction_status_history.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	transaction_status_history := new(model.Transaction_status_history)
	errBind := c.Bind(transaction_status_history)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateTransaction_status_history(c.Request().Context(), transaction_status_history, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateTransaction_status_history.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

