package wallet_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  wallet_typesSV "palm-pay/app/wallet_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF wallet_typesSV.Wallet_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewWallet_typesHandler(service wallet_typesSV.Wallet_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("wallet_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetWallet_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_types.Handle -> handler.GetWallet_typess.GetWallet_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetWallet_typess.Limit", limitParam)
	tracker.AddParam("handler.GetWallet_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetWallet_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Wallet_types); ok {
		tracker.AddResult("handler.GetWallet_typess.Count", len(items))
		tracker.AddResult("handler.GetWallet_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetWallet_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_types.Handle -> GetWallet_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetWallet_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetWallet_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetWallet_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetWallet_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetWallet_typesByTypeCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_typesHandle -> GetWallet_typesByTypeCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetWallet_typesByTypeCode")
	defer tracker.Finish(&err)

	typecodeParam := c.Param("typecode")

	typecode := typecodeParam

	result, err := h.serviceIF.GetWallet_typesByTypeCode(c.Request().Context(), typecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetWallet_typesByTypeCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteWallet_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_typesHandle -> DeleteWallet_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteWallet_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteWallet_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteWallet_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteWallet_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteWallet_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertWallet_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_typesHandle -> InsertWallet_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertWallet_types")
	defer tracker.Finish(&err)




	wallet_types := new(model.Wallet_types)
	errBind := c.Bind(wallet_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertWallet_types(c.Request().Context(), wallet_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertWallet_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateWallet_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Wallet_typesHandle -> UpdateWallet_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateWallet_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateWallet_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	wallet_types := new(model.Wallet_types)
	errBind := c.Bind(wallet_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateWallet_types(c.Request().Context(), wallet_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateWallet_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

