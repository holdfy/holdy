package notification_channelsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  notification_channelsSV "palm-pay/app/notification_channels/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF notification_channelsSV.Notification_channelsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewNotification_channelsHandler(service notification_channelsSV.Notification_channelsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("notification_channels"), // <---- adicionado aqui
     }
}
func (h Handler)  GetNotification_channels(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_channels.Handle -> handler.GetNotification_channelss.GetNotification_channels", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_channelss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetNotification_channelss.Limit", limitParam)
	tracker.AddParam("handler.GetNotification_channelss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetNotification_channels(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Notification_channels); ok {
		tracker.AddResult("handler.GetNotification_channelss.Count", len(items))
		tracker.AddResult("handler.GetNotification_channelss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetNotification_channelsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_channels.Handle -> GetNotification_channelsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_channelsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetNotification_channelsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetNotification_channelsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetNotification_channelsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetNotification_channelsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetNotification_channelsByChannelCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_channelsHandle -> GetNotification_channelsByChannelCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_channelsByChannelCode")
	defer tracker.Finish(&err)

	channelcodeParam := c.Param("channelcode")

	channelcode := channelcodeParam

	result, err := h.serviceIF.GetNotification_channelsByChannelCode(c.Request().Context(), channelcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetNotification_channelsByChannelCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteNotification_channelsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_channelsHandle -> DeleteNotification_channelsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteNotification_channelsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteNotification_channelsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteNotification_channelsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteNotification_channelsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteNotification_channelsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertNotification_channels(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_channelsHandle -> InsertNotification_channels", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertNotification_channels")
	defer tracker.Finish(&err)




	notification_channels := new(model.Notification_channels)
	errBind := c.Bind(notification_channels)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertNotification_channels(c.Request().Context(), notification_channels)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertNotification_channels.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateNotification_channels(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_channelsHandle -> UpdateNotification_channels", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateNotification_channels")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateNotification_channels.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	notification_channels := new(model.Notification_channels)
	errBind := c.Bind(notification_channels)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateNotification_channels(c.Request().Context(), notification_channels, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateNotification_channels.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

