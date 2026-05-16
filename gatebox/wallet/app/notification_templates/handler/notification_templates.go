package notification_templatesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  notification_templatesSV "palm-pay/app/notification_templates/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF notification_templatesSV.Notification_templatesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewNotification_templatesHandler(service notification_templatesSV.Notification_templatesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("notification_templates"), // <---- adicionado aqui
     }
}
func (h Handler)  GetNotification_templates(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_templates.Handle -> handler.GetNotification_templatess.GetNotification_templates", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_templatess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetNotification_templatess.Limit", limitParam)
	tracker.AddParam("handler.GetNotification_templatess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetNotification_templates(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Notification_templates); ok {
		tracker.AddResult("handler.GetNotification_templatess.Count", len(items))
		tracker.AddResult("handler.GetNotification_templatess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetNotification_templatesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_templates.Handle -> GetNotification_templatesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_templatesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetNotification_templatesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetNotification_templatesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetNotification_templatesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetNotification_templatesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetNotification_templatesByTemplateCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_templatesHandle -> GetNotification_templatesByTemplateCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetNotification_templatesByTemplateCode")
	defer tracker.Finish(&err)

	templatecodeParam := c.Param("templatecode")

	templatecode := templatecodeParam

	result, err := h.serviceIF.GetNotification_templatesByTemplateCode(c.Request().Context(), templatecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetNotification_templatesByTemplateCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteNotification_templatesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_templatesHandle -> DeleteNotification_templatesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteNotification_templatesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteNotification_templatesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteNotification_templatesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteNotification_templatesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteNotification_templatesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertNotification_templates(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_templatesHandle -> InsertNotification_templates", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertNotification_templates")
	defer tracker.Finish(&err)




	notification_templates := new(model.Notification_templates)
	errBind := c.Bind(notification_templates)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertNotification_templates(c.Request().Context(), notification_templates)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertNotification_templates.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateNotification_templates(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Notification_templatesHandle -> UpdateNotification_templates", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateNotification_templates")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateNotification_templates.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	notification_templates := new(model.Notification_templates)
	errBind := c.Bind(notification_templates)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateNotification_templates(c.Request().Context(), notification_templates, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateNotification_templates.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

