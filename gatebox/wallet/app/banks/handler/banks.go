package banksHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  banksSV "palm-pay/app/banks/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF banksSV.BanksServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewBanksHandler(service banksSV.BanksServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("banks"), // <---- adicionado aqui
     }
}
func (h Handler)  GetBanks(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Banks.Handle -> handler.GetBankss.GetBanks", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetBankss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetBankss.Limit", limitParam)
	tracker.AddParam("handler.GetBankss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetBanks(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Banks); ok {
		tracker.AddResult("handler.GetBankss.Count", len(items))
		tracker.AddResult("handler.GetBankss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetBanksById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Banks.Handle -> GetBanksById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetBanksById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetBanksById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetBanksById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetBanksById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetBanksById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetBanksByBankCodeInternal(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("BanksHandle -> GetBanksByBankCodeInternal", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetBanksByBankCodeInternal")
	defer tracker.Finish(&err)

	bankcodeinternalParam := c.Param("bankcodeinternal")

	bankcodeinternal := bankcodeinternalParam

	result, err := h.serviceIF.GetBanksByBankCodeInternal(c.Request().Context(), bankcodeinternal)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetBanksByBankCodeInternal.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteBanksById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("BanksHandle -> DeleteBanksById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteBanksById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteBanksById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteBanksById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteBanksById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteBanksById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertBanks(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("BanksHandle -> InsertBanks", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertBanks")
	defer tracker.Finish(&err)




	banks := new(model.Banks)
	errBind := c.Bind(banks)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertBanks(c.Request().Context(), banks)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertBanks.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateBanks(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("BanksHandle -> UpdateBanks", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateBanks")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateBanks.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	banks := new(model.Banks)
	errBind := c.Bind(banks)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateBanks(c.Request().Context(), banks, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateBanks.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

