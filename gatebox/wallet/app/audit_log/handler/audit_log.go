package audit_logHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  audit_logSV "palm-pay/app/audit_log/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF audit_logSV.Audit_logServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewAudit_logHandler(service audit_logSV.Audit_logServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("audit_log"), // <---- adicionado aqui
     }
}
func (h Handler)  GetAudit_log(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_log.Handle -> handler.GetAudit_logs.GetAudit_log", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAudit_logs")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetAudit_logs.Limit", limitParam)
	tracker.AddParam("handler.GetAudit_logs.Offset", offsetParam)
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

	result, err := h.serviceIF.GetAudit_log(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Audit_log); ok {
		tracker.AddResult("handler.GetAudit_logs.Count", len(items))
		tracker.AddResult("handler.GetAudit_logs.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetAudit_logById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_log.Handle -> GetAudit_logById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAudit_logById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetAudit_logById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetAudit_logById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAudit_logById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetAudit_logById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetAudit_logByAuditCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_logHandle -> GetAudit_logByAuditCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAudit_logByAuditCode")
	defer tracker.Finish(&err)

	auditcodeParam := c.Param("auditcode")

	auditcode := auditcodeParam

	result, err := h.serviceIF.GetAudit_logByAuditCode(c.Request().Context(), auditcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAudit_logByAuditCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteAudit_logById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_logHandle -> DeleteAudit_logById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteAudit_logById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteAudit_logById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteAudit_logById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteAudit_logById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteAudit_logById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertAudit_log(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_logHandle -> InsertAudit_log", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertAudit_log")
	defer tracker.Finish(&err)




	audit_log := new(model.Audit_log)
	errBind := c.Bind(audit_log)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertAudit_log(c.Request().Context(), audit_log)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertAudit_log.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateAudit_log(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_logHandle -> UpdateAudit_log", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateAudit_log")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateAudit_log.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	audit_log := new(model.Audit_log)
	errBind := c.Bind(audit_log)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateAudit_log(c.Request().Context(), audit_log, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateAudit_log.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

