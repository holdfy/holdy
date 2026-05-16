package transactionsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  transactionsSV "palm-pay/app/transactions/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF transactionsSV.TransactionsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewTransactionsHandler(service transactionsSV.TransactionsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("transactions"), // <---- adicionado aqui
     }
}
func (h Handler)  GetTransactions(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Transactions.Handle -> handler.GetTransactionss.GetTransactions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransactionss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetTransactionss.Limit", limitParam)
	tracker.AddParam("handler.GetTransactionss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetTransactions(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Transactions); ok {
		tracker.AddResult("handler.GetTransactionss.Count", len(items))
		tracker.AddResult("handler.GetTransactionss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetTransactionsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transactions.Handle -> GetTransactionsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransactionsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetTransactionsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetTransactionsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetTransactionsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetTransactionsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetTransactionsByTransactionCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("TransactionsHandle -> GetTransactionsByTransactionCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransactionsByTransactionCode")
	defer tracker.Finish(&err)

	transactioncodeParam := c.Param("transactioncode")

	transactioncode := transactioncodeParam

	result, err := h.serviceIF.GetTransactionsByTransactionCode(c.Request().Context(), transactioncode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetTransactionsByTransactionCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteTransactionsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("TransactionsHandle -> DeleteTransactionsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteTransactionsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteTransactionsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteTransactionsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteTransactionsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteTransactionsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertTransactions(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("TransactionsHandle -> InsertTransactions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertTransactions")
	defer tracker.Finish(&err)




	transactions := new(model.Transactions)
	errBind := c.Bind(transactions)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertTransactions(c.Request().Context(), transactions)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertTransactions.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateTransactions(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("TransactionsHandle -> UpdateTransactions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateTransactions")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateTransactions.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	transactions := new(model.Transactions)
	errBind := c.Bind(transactions)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateTransactions(c.Request().Context(), transactions, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateTransactions.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

