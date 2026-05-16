package acquirersHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  acquirersSV "palm-pay/app/acquirers/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF acquirersSV.AcquirersServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewAcquirersHandler(service acquirersSV.AcquirersServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("acquirers"), // <---- adicionado aqui
     }
}
func (h Handler)  GetAcquirers(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Acquirers.Handle -> handler.GetAcquirerss.GetAcquirers", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAcquirerss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetAcquirerss.Limit", limitParam)
	tracker.AddParam("handler.GetAcquirerss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetAcquirers(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Acquirers); ok {
		tracker.AddResult("handler.GetAcquirerss.Count", len(items))
		tracker.AddResult("handler.GetAcquirerss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetAcquirersById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Acquirers.Handle -> GetAcquirersById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAcquirersById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetAcquirersById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetAcquirersById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAcquirersById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetAcquirersById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetAcquirersByAcquirerCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("AcquirersHandle -> GetAcquirersByAcquirerCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAcquirersByAcquirerCode")
	defer tracker.Finish(&err)

	acquirercodeParam := c.Param("acquirercode")

	acquirercode := acquirercodeParam

	result, err := h.serviceIF.GetAcquirersByAcquirerCode(c.Request().Context(), acquirercode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAcquirersByAcquirerCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteAcquirersById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("AcquirersHandle -> DeleteAcquirersById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteAcquirersById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteAcquirersById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteAcquirersById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteAcquirersById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteAcquirersById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertAcquirers(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("AcquirersHandle -> InsertAcquirers", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertAcquirers")
	defer tracker.Finish(&err)




	acquirers := new(model.Acquirers)
	errBind := c.Bind(acquirers)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertAcquirers(c.Request().Context(), acquirers)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertAcquirers.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateAcquirers(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("AcquirersHandle -> UpdateAcquirers", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateAcquirers")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateAcquirers.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	acquirers := new(model.Acquirers)
	errBind := c.Bind(acquirers)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateAcquirers(c.Request().Context(), acquirers, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateAcquirers.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

