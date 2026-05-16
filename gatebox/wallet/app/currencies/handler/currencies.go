package currenciesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  currenciesSV "palm-pay/app/currencies/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF currenciesSV.CurrenciesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewCurrenciesHandler(service currenciesSV.CurrenciesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("currencies"), // <---- adicionado aqui
     }
}
func (h Handler)  GetCurrencies(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Currencies.Handle -> handler.GetCurrenciess.GetCurrencies", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetCurrenciess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetCurrenciess.Limit", limitParam)
	tracker.AddParam("handler.GetCurrenciess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetCurrencies(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Currencies); ok {
		tracker.AddResult("handler.GetCurrenciess.Count", len(items))
		tracker.AddResult("handler.GetCurrenciess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetCurrenciesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Currencies.Handle -> GetCurrenciesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetCurrenciesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetCurrenciesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetCurrenciesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetCurrenciesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetCurrenciesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetCurrenciesByCurrencyCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("CurrenciesHandle -> GetCurrenciesByCurrencyCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetCurrenciesByCurrencyCode")
	defer tracker.Finish(&err)

	currencycodeParam := c.Param("currencycode")

	currencycode := currencycodeParam

	result, err := h.serviceIF.GetCurrenciesByCurrencyCode(c.Request().Context(), currencycode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetCurrenciesByCurrencyCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteCurrenciesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("CurrenciesHandle -> DeleteCurrenciesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteCurrenciesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteCurrenciesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteCurrenciesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteCurrenciesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteCurrenciesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertCurrencies(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("CurrenciesHandle -> InsertCurrencies", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertCurrencies")
	defer tracker.Finish(&err)




	currencies := new(model.Currencies)
	errBind := c.Bind(currencies)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertCurrencies(c.Request().Context(), currencies)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertCurrencies.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateCurrencies(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("CurrenciesHandle -> UpdateCurrencies", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateCurrencies")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateCurrencies.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	currencies := new(model.Currencies)
	errBind := c.Bind(currencies)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateCurrencies(c.Request().Context(), currencies, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateCurrencies.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

