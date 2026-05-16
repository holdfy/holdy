package notification_historyHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  notification_historySV "palm-pay/app/notification_history/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF notification_historySV.Notification_historyServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewNotification_historyHandler(service notification_historySV.Notification_historyServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("notification_history"), // <---- adicionado aqui
     }
}
func (h Handler)  GetNotification_history(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_history.Handle -> handler.GetNotification_historys.GetNotification_history", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_historys")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetNotification_historys.Limit", limitParam)
	tracker.AddParam("handler.GetNotification_historys.Offset", offsetParam)
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

	result, err := h.serviceIF.GetNotification_history(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Notification_history); ok {
		tracker.AddResult("handler.GetNotification_historys.Count", len(items))
		tracker.AddResult("handler.GetNotification_historys.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetNotification_historyById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_history.Handle -> GetNotification_historyById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_historyById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetNotification_historyById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetNotification_historyById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetNotification_historyById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetNotification_historyById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetNotification_historyByNotificationCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_historyHandle -> GetNotification_historyByNotificationCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_historyByNotificationCode")
	defer tracker.Finish(&err)

	notificationcodeParam := c.Param("notificationcode")

	notificationcode := notificationcodeParam

	result, err := h.serviceIF.GetNotification_historyByNotificationCode(c.Request().Context(), notificationcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetNotification_historyByNotificationCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteNotification_historyById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_historyHandle -> DeleteNotification_historyById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteNotification_historyById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteNotification_historyById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteNotification_historyById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteNotification_historyById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteNotification_historyById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertNotification_history(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_historyHandle -> InsertNotification_history", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertNotification_history")
	defer tracker.Finish(&err)




	notification_history := new(model.Notification_history)
	errBind := c.Bind(notification_history)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertNotification_history(c.Request().Context(), notification_history)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertNotification_history.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateNotification_history(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_historyHandle -> UpdateNotification_history", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateNotification_history")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateNotification_history.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	notification_history := new(model.Notification_history)
	errBind := c.Bind(notification_history)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateNotification_history(c.Request().Context(), notification_history, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateNotification_history.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

