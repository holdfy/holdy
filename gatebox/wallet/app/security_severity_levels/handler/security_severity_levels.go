package security_severity_levelsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  security_severity_levelsSV "palm-pay/app/security_severity_levels/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF security_severity_levelsSV.Security_severity_levelsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewSecurity_severity_levelsHandler(service security_severity_levelsSV.Security_severity_levelsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("security_severity_levels"), // <---- adicionado aqui
     }
}
func (h Handler)  GetSecurity_severity_levels(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_severity_levels.Handle -> handler.GetSecurity_severity_levelss.GetSecurity_severity_levels", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSecurity_severity_levelss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetSecurity_severity_levelss.Limit", limitParam)
	tracker.AddParam("handler.GetSecurity_severity_levelss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetSecurity_severity_levels(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Security_severity_levels); ok {
		tracker.AddResult("handler.GetSecurity_severity_levelss.Count", len(items))
		tracker.AddResult("handler.GetSecurity_severity_levelss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetSecurity_severity_levelsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_severity_levels.Handle -> GetSecurity_severity_levelsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSecurity_severity_levelsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetSecurity_severity_levelsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetSecurity_severity_levelsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetSecurity_severity_levelsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetSecurity_severity_levelsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetSecurity_severity_levelsBySeverityCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_severity_levelsHandle -> GetSecurity_severity_levelsBySeverityCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSecurity_severity_levelsBySeverityCode")
	defer tracker.Finish(&err)

	severitycodeParam := c.Param("severitycode")

	severitycode := severitycodeParam

	result, err := h.serviceIF.GetSecurity_severity_levelsBySeverityCode(c.Request().Context(), severitycode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetSecurity_severity_levelsBySeverityCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteSecurity_severity_levelsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_severity_levelsHandle -> DeleteSecurity_severity_levelsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteSecurity_severity_levelsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteSecurity_severity_levelsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteSecurity_severity_levelsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteSecurity_severity_levelsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteSecurity_severity_levelsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertSecurity_severity_levels(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_severity_levelsHandle -> InsertSecurity_severity_levels", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertSecurity_severity_levels")
	defer tracker.Finish(&err)




	security_severity_levels := new(model.Security_severity_levels)
	errBind := c.Bind(security_severity_levels)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertSecurity_severity_levels(c.Request().Context(), security_severity_levels)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertSecurity_severity_levels.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateSecurity_severity_levels(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Security_severity_levelsHandle -> UpdateSecurity_severity_levels", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateSecurity_severity_levels")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateSecurity_severity_levels.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	security_severity_levels := new(model.Security_severity_levels)
	errBind := c.Bind(security_severity_levels)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateSecurity_severity_levels(c.Request().Context(), security_severity_levels, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateSecurity_severity_levels.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

