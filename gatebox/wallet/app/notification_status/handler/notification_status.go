package notification_statusHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  notification_statusSV "palm-pay/app/notification_status/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF notification_statusSV.Notification_statusServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewNotification_statusHandler(service notification_statusSV.Notification_statusServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("notification_status"), // <---- adicionado aqui
     }
}
func (h Handler)  GetNotification_status(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_status.Handle -> handler.GetNotification_statuss.GetNotification_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_statuss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetNotification_statuss.Limit", limitParam)
	tracker.AddParam("handler.GetNotification_statuss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetNotification_status(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Notification_status); ok {
		tracker.AddResult("handler.GetNotification_statuss.Count", len(items))
		tracker.AddResult("handler.GetNotification_statuss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetNotification_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_status.Handle -> GetNotification_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetNotification_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetNotification_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetNotification_statusById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetNotification_statusById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetNotification_statusByStatusCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_statusHandle -> GetNotification_statusByStatusCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_statusByStatusCode")
	defer tracker.Finish(&err)

	statuscodeParam := c.Param("statuscode")

	statuscode := statuscodeParam

	result, err := h.serviceIF.GetNotification_statusByStatusCode(c.Request().Context(), statuscode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetNotification_statusByStatusCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteNotification_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_statusHandle -> DeleteNotification_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteNotification_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteNotification_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteNotification_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteNotification_statusById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteNotification_statusById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertNotification_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_statusHandle -> InsertNotification_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertNotification_status")
	defer tracker.Finish(&err)




	notification_status := new(model.Notification_status)
	errBind := c.Bind(notification_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertNotification_status(c.Request().Context(), notification_status)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertNotification_status.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateNotification_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_statusHandle -> UpdateNotification_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateNotification_status")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateNotification_status.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	notification_status := new(model.Notification_status)
	errBind := c.Bind(notification_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateNotification_status(c.Request().Context(), notification_status, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateNotification_status.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

