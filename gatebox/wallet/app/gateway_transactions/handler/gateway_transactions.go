package gateway_transactionsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  gateway_transactionsSV "palm-pay/app/gateway_transactions/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF gateway_transactionsSV.Gateway_transactionsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewGateway_transactionsHandler(service gateway_transactionsSV.Gateway_transactionsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("gateway_transactions"), // <---- adicionado aqui
     }
}
func (h Handler)  GetGateway_transactions(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_transactions.Handle -> handler.GetGateway_transactionss.GetGateway_transactions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetGateway_transactionss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetGateway_transactionss.Limit", limitParam)
	tracker.AddParam("handler.GetGateway_transactionss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetGateway_transactions(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Gateway_transactions); ok {
		tracker.AddResult("handler.GetGateway_transactionss.Count", len(items))
		tracker.AddResult("handler.GetGateway_transactionss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetGateway_transactionsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_transactions.Handle -> GetGateway_transactionsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetGateway_transactionsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetGateway_transactionsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetGateway_transactionsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetGateway_transactionsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetGateway_transactionsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetGateway_transactionsByGatewayTransactionCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_transactionsHandle -> GetGateway_transactionsByGatewayTransactionCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetGateway_transactionsByGatewayTransactionCode")
	defer tracker.Finish(&err)

	gatewaytransactioncodeParam := c.Param("gatewaytransactioncode")

	gatewaytransactioncode := gatewaytransactioncodeParam

	result, err := h.serviceIF.GetGateway_transactionsByGatewayTransactionCode(c.Request().Context(), gatewaytransactioncode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetGateway_transactionsByGatewayTransactionCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteGateway_transactionsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_transactionsHandle -> DeleteGateway_transactionsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteGateway_transactionsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteGateway_transactionsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteGateway_transactionsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteGateway_transactionsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteGateway_transactionsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertGateway_transactions(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_transactionsHandle -> InsertGateway_transactions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertGateway_transactions")
	defer tracker.Finish(&err)




	gateway_transactions := new(model.Gateway_transactions)
	errBind := c.Bind(gateway_transactions)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertGateway_transactions(c.Request().Context(), gateway_transactions)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertGateway_transactions.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateGateway_transactions(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_transactionsHandle -> UpdateGateway_transactions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateGateway_transactions")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateGateway_transactions.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	gateway_transactions := new(model.Gateway_transactions)
	errBind := c.Bind(gateway_transactions)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateGateway_transactions(c.Request().Context(), gateway_transactions, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateGateway_transactions.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

