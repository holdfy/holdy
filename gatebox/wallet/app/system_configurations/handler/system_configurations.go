package system_configurationsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  system_configurationsSV "palm-pay/app/system_configurations/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF system_configurationsSV.System_configurationsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewSystem_configurationsHandler(service system_configurationsSV.System_configurationsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("system_configurations"), // <---- adicionado aqui
     }
}
func (h Handler)  GetSystem_configurations(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("System_configurations.Handle -> handler.GetSystem_configurationss.GetSystem_configurations", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSystem_configurationss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetSystem_configurationss.Limit", limitParam)
	tracker.AddParam("handler.GetSystem_configurationss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetSystem_configurations(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.System_configurations); ok {
		tracker.AddResult("handler.GetSystem_configurationss.Count", len(items))
		tracker.AddResult("handler.GetSystem_configurationss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetSystem_configurationsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("System_configurations.Handle -> GetSystem_configurationsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSystem_configurationsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetSystem_configurationsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetSystem_configurationsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetSystem_configurationsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetSystem_configurationsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetSystem_configurationsByConfigCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("System_configurationsHandle -> GetSystem_configurationsByConfigCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSystem_configurationsByConfigCode")
	defer tracker.Finish(&err)

	configcodeParam := c.Param("configcode")

	configcode := configcodeParam

	result, err := h.serviceIF.GetSystem_configurationsByConfigCode(c.Request().Context(), configcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetSystem_configurationsByConfigCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteSystem_configurationsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("System_configurationsHandle -> DeleteSystem_configurationsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteSystem_configurationsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteSystem_configurationsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteSystem_configurationsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteSystem_configurationsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteSystem_configurationsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertSystem_configurations(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("System_configurationsHandle -> InsertSystem_configurations", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertSystem_configurations")
	defer tracker.Finish(&err)




	system_configurations := new(model.System_configurations)
	errBind := c.Bind(system_configurations)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertSystem_configurations(c.Request().Context(), system_configurations)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertSystem_configurations.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateSystem_configurations(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("System_configurationsHandle -> UpdateSystem_configurations", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateSystem_configurations")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateSystem_configurations.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	system_configurations := new(model.System_configurations)
	errBind := c.Bind(system_configurations)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateSystem_configurations(c.Request().Context(), system_configurations, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateSystem_configurations.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

