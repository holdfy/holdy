package kyc_statusHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  kyc_statusSV "palm-pay/app/kyc_status/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF kyc_statusSV.Kyc_statusServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewKyc_statusHandler(service kyc_statusSV.Kyc_statusServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("kyc_status"), // <---- adicionado aqui
     }
}
func (h Handler)  GetKyc_status(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Kyc_status.Handle -> handler.GetKyc_statuss.GetKyc_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetKyc_statuss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetKyc_statuss.Limit", limitParam)
	tracker.AddParam("handler.GetKyc_statuss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetKyc_status(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Kyc_status); ok {
		tracker.AddResult("handler.GetKyc_statuss.Count", len(items))
		tracker.AddResult("handler.GetKyc_statuss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetKyc_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Kyc_status.Handle -> GetKyc_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetKyc_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetKyc_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetKyc_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetKyc_statusById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetKyc_statusById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetKyc_statusByStatusCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Kyc_statusHandle -> GetKyc_statusByStatusCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetKyc_statusByStatusCode")
	defer tracker.Finish(&err)

	statuscodeParam := c.Param("statuscode")

	statuscode := statuscodeParam

	result, err := h.serviceIF.GetKyc_statusByStatusCode(c.Request().Context(), statuscode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetKyc_statusByStatusCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteKyc_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Kyc_statusHandle -> DeleteKyc_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteKyc_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteKyc_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteKyc_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteKyc_statusById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteKyc_statusById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertKyc_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Kyc_statusHandle -> InsertKyc_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertKyc_status")
	defer tracker.Finish(&err)




	kyc_status := new(model.Kyc_status)
	errBind := c.Bind(kyc_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertKyc_status(c.Request().Context(), kyc_status)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertKyc_status.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateKyc_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Kyc_statusHandle -> UpdateKyc_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateKyc_status")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateKyc_status.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	kyc_status := new(model.Kyc_status)
	errBind := c.Bind(kyc_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateKyc_status(c.Request().Context(), kyc_status, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateKyc_status.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

