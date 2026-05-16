package wallet_balance_historyHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  wallet_balance_historySV "palm-pay/app/wallet_balance_history/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF wallet_balance_historySV.Wallet_balance_historyServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewWallet_balance_historyHandler(service wallet_balance_historySV.Wallet_balance_historyServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("wallet_balance_history"), // <---- adicionado aqui
     }
}
func (h Handler)  GetWallet_balance_history(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_balance_history.Handle -> handler.GetWallet_balance_historys.GetWallet_balance_history", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_balance_historys")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetWallet_balance_historys.Limit", limitParam)
	tracker.AddParam("handler.GetWallet_balance_historys.Offset", offsetParam)
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

	result, err := h.serviceIF.GetWallet_balance_history(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Wallet_balance_history); ok {
		tracker.AddResult("handler.GetWallet_balance_historys.Count", len(items))
		tracker.AddResult("handler.GetWallet_balance_historys.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetWallet_balance_historyById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_balance_history.Handle -> GetWallet_balance_historyById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_balance_historyById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetWallet_balance_historyById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetWallet_balance_historyById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetWallet_balance_historyById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetWallet_balance_historyById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetWallet_balance_historyByBalanceHistoryCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_balance_historyHandle -> GetWallet_balance_historyByBalanceHistoryCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_balance_historyByBalanceHistoryCode")
	defer tracker.Finish(&err)

	balancehistorycodeParam := c.Param("balancehistorycode")

	balancehistorycode := balancehistorycodeParam

	result, err := h.serviceIF.GetWallet_balance_historyByBalanceHistoryCode(c.Request().Context(), balancehistorycode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetWallet_balance_historyByBalanceHistoryCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteWallet_balance_historyById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_balance_historyHandle -> DeleteWallet_balance_historyById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteWallet_balance_historyById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteWallet_balance_historyById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteWallet_balance_historyById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteWallet_balance_historyById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteWallet_balance_historyById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertWallet_balance_history(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_balance_historyHandle -> InsertWallet_balance_history", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertWallet_balance_history")
	defer tracker.Finish(&err)




	wallet_balance_history := new(model.Wallet_balance_history)
	errBind := c.Bind(wallet_balance_history)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertWallet_balance_history(c.Request().Context(), wallet_balance_history)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertWallet_balance_history.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateWallet_balance_history(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_balance_historyHandle -> UpdateWallet_balance_history", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateWallet_balance_history")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateWallet_balance_history.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	wallet_balance_history := new(model.Wallet_balance_history)
	errBind := c.Bind(wallet_balance_history)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateWallet_balance_history(c.Request().Context(), wallet_balance_history, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateWallet_balance_history.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

