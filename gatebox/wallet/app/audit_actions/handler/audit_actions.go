package audit_actionsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  audit_actionsSV "palm-pay/app/audit_actions/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF audit_actionsSV.Audit_actionsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewAudit_actionsHandler(service audit_actionsSV.Audit_actionsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("audit_actions"), // <---- adicionado aqui
     }
}
func (h Handler)  GetAudit_actions(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_actions.Handle -> handler.GetAudit_actionss.GetAudit_actions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAudit_actionss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetAudit_actionss.Limit", limitParam)
	tracker.AddParam("handler.GetAudit_actionss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetAudit_actions(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Audit_actions); ok {
		tracker.AddResult("handler.GetAudit_actionss.Count", len(items))
		tracker.AddResult("handler.GetAudit_actionss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetAudit_actionsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_actions.Handle -> GetAudit_actionsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAudit_actionsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetAudit_actionsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetAudit_actionsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAudit_actionsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetAudit_actionsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetAudit_actionsByActionCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_actionsHandle -> GetAudit_actionsByActionCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAudit_actionsByActionCode")
	defer tracker.Finish(&err)

	actioncodeParam := c.Param("actioncode")

	actioncode := actioncodeParam

	result, err := h.serviceIF.GetAudit_actionsByActionCode(c.Request().Context(), actioncode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAudit_actionsByActionCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteAudit_actionsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_actionsHandle -> DeleteAudit_actionsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteAudit_actionsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteAudit_actionsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteAudit_actionsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteAudit_actionsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteAudit_actionsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertAudit_actions(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_actionsHandle -> InsertAudit_actions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertAudit_actions")
	defer tracker.Finish(&err)




	audit_actions := new(model.Audit_actions)
	errBind := c.Bind(audit_actions)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertAudit_actions(c.Request().Context(), audit_actions)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertAudit_actions.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateAudit_actions(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_actionsHandle -> UpdateAudit_actions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateAudit_actions")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateAudit_actions.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	audit_actions := new(model.Audit_actions)
	errBind := c.Bind(audit_actions)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateAudit_actions(c.Request().Context(), audit_actions, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateAudit_actions.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

