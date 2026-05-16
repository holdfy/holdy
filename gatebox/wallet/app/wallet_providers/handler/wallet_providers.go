package wallet_providersHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  wallet_providersSV "palm-pay/app/wallet_providers/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF wallet_providersSV.Wallet_providersServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewWallet_providersHandler(service wallet_providersSV.Wallet_providersServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("wallet_providers"), // <---- adicionado aqui
     }
}
func (h Handler)  GetWallet_providers(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_providers.Handle -> handler.GetWallet_providerss.GetWallet_providers", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_providerss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetWallet_providerss.Limit", limitParam)
	tracker.AddParam("handler.GetWallet_providerss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetWallet_providers(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Wallet_providers); ok {
		tracker.AddResult("handler.GetWallet_providerss.Count", len(items))
		tracker.AddResult("handler.GetWallet_providerss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetWallet_providersById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_providers.Handle -> GetWallet_providersById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_providersById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetWallet_providersById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetWallet_providersById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetWallet_providersById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetWallet_providersById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetWallet_providersByProviderCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_providersHandle -> GetWallet_providersByProviderCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_providersByProviderCode")
	defer tracker.Finish(&err)

	providercodeParam := c.Param("providercode")

	providercode := providercodeParam

	result, err := h.serviceIF.GetWallet_providersByProviderCode(c.Request().Context(), providercode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetWallet_providersByProviderCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteWallet_providersById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_providersHandle -> DeleteWallet_providersById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteWallet_providersById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteWallet_providersById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteWallet_providersById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteWallet_providersById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteWallet_providersById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertWallet_providers(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_providersHandle -> InsertWallet_providers", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertWallet_providers")
	defer tracker.Finish(&err)




	wallet_providers := new(model.Wallet_providers)
	errBind := c.Bind(wallet_providers)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertWallet_providers(c.Request().Context(), wallet_providers)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertWallet_providers.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateWallet_providers(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_providersHandle -> UpdateWallet_providers", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateWallet_providers")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateWallet_providers.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	wallet_providers := new(model.Wallet_providers)
	errBind := c.Bind(wallet_providers)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateWallet_providers(c.Request().Context(), wallet_providers, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateWallet_providers.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

