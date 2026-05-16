package external_walletsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  external_walletsSV "palm-pay/app/external_wallets/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF external_walletsSV.External_walletsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewExternal_walletsHandler(service external_walletsSV.External_walletsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("external_wallets"), // <---- adicionado aqui
     }
}
func (h Handler)  GetExternal_wallets(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("External_wallets.Handle -> handler.GetExternal_walletss.GetExternal_wallets", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetExternal_walletss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetExternal_walletss.Limit", limitParam)
	tracker.AddParam("handler.GetExternal_walletss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetExternal_wallets(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.External_wallets); ok {
		tracker.AddResult("handler.GetExternal_walletss.Count", len(items))
		tracker.AddResult("handler.GetExternal_walletss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetExternal_walletsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("External_wallets.Handle -> GetExternal_walletsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetExternal_walletsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetExternal_walletsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetExternal_walletsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetExternal_walletsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetExternal_walletsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetExternal_walletsByExternalWalletCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("External_walletsHandle -> GetExternal_walletsByExternalWalletCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetExternal_walletsByExternalWalletCode")
	defer tracker.Finish(&err)

	externalwalletcodeParam := c.Param("externalwalletcode")

	externalwalletcode := externalwalletcodeParam

	result, err := h.serviceIF.GetExternal_walletsByExternalWalletCode(c.Request().Context(), externalwalletcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetExternal_walletsByExternalWalletCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteExternal_walletsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("External_walletsHandle -> DeleteExternal_walletsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteExternal_walletsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteExternal_walletsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteExternal_walletsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteExternal_walletsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteExternal_walletsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertExternal_wallets(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("External_walletsHandle -> InsertExternal_wallets", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertExternal_wallets")
	defer tracker.Finish(&err)




	external_wallets := new(model.External_wallets)
	errBind := c.Bind(external_wallets)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertExternal_wallets(c.Request().Context(), external_wallets)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertExternal_wallets.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateExternal_wallets(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("External_walletsHandle -> UpdateExternal_wallets", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateExternal_wallets")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateExternal_wallets.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	external_wallets := new(model.External_wallets)
	errBind := c.Bind(external_wallets)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateExternal_wallets(c.Request().Context(), external_wallets, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateExternal_wallets.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

