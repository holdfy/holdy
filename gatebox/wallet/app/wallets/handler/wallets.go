package walletsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  walletsSV "palm-pay/app/wallets/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF walletsSV.WalletsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewWalletsHandler(service walletsSV.WalletsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("wallets"), // <---- adicionado aqui
     }
}
func (h Handler)  GetWallets(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallets.Handle -> handler.GetWalletss.GetWallets", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWalletss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetWalletss.Limit", limitParam)
	tracker.AddParam("handler.GetWalletss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetWallets(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Wallets); ok {
		tracker.AddResult("handler.GetWalletss.Count", len(items))
		tracker.AddResult("handler.GetWalletss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetWalletsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallets.Handle -> GetWalletsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWalletsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetWalletsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetWalletsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetWalletsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetWalletsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetWalletsByWalletCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("WalletsHandle -> GetWalletsByWalletCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWalletsByWalletCode")
	defer tracker.Finish(&err)

	walletcodeParam := c.Param("walletcode")

	walletcode := walletcodeParam

	result, err := h.serviceIF.GetWalletsByWalletCode(c.Request().Context(), walletcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetWalletsByWalletCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteWalletsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("WalletsHandle -> DeleteWalletsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteWalletsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteWalletsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteWalletsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteWalletsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteWalletsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertWallets(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("WalletsHandle -> InsertWallets", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertWallets")
	defer tracker.Finish(&err)




	wallets := new(model.Wallets)
	errBind := c.Bind(wallets)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertWallets(c.Request().Context(), wallets)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertWallets.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateWallets(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("WalletsHandle -> UpdateWallets", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateWallets")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateWallets.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	wallets := new(model.Wallets)
	errBind := c.Bind(wallets)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateWallets(c.Request().Context(), wallets, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateWallets.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

