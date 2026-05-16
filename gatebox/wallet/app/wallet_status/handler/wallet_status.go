package wallet_statusHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  wallet_statusSV "palm-pay/app/wallet_status/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF wallet_statusSV.Wallet_statusServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewWallet_statusHandler(service wallet_statusSV.Wallet_statusServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("wallet_status"), // <---- adicionado aqui
     }
}
func (h Handler)  GetWallet_status(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_status.Handle -> handler.GetWallet_statuss.GetWallet_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_statuss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetWallet_statuss.Limit", limitParam)
	tracker.AddParam("handler.GetWallet_statuss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetWallet_status(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Wallet_status); ok {
		tracker.AddResult("handler.GetWallet_statuss.Count", len(items))
		tracker.AddResult("handler.GetWallet_statuss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetWallet_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_status.Handle -> GetWallet_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetWallet_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetWallet_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetWallet_statusById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetWallet_statusById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetWallet_statusByStatusCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_statusHandle -> GetWallet_statusByStatusCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_statusByStatusCode")
	defer tracker.Finish(&err)

	statuscodeParam := c.Param("statuscode")

	statuscode := statuscodeParam

	result, err := h.serviceIF.GetWallet_statusByStatusCode(c.Request().Context(), statuscode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetWallet_statusByStatusCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteWallet_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_statusHandle -> DeleteWallet_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteWallet_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteWallet_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteWallet_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteWallet_statusById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteWallet_statusById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertWallet_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_statusHandle -> InsertWallet_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertWallet_status")
	defer tracker.Finish(&err)




	wallet_status := new(model.Wallet_status)
	errBind := c.Bind(wallet_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertWallet_status(c.Request().Context(), wallet_status)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertWallet_status.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateWallet_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_statusHandle -> UpdateWallet_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateWallet_status")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateWallet_status.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	wallet_status := new(model.Wallet_status)
	errBind := c.Bind(wallet_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateWallet_status(c.Request().Context(), wallet_status, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateWallet_status.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

