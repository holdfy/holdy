package audit_tablesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  audit_tablesSV "palm-pay/app/audit_tables/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF audit_tablesSV.Audit_tablesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewAudit_tablesHandler(service audit_tablesSV.Audit_tablesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("audit_tables"), // <---- adicionado aqui
     }
}
func (h Handler)  GetAudit_tables(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_tables.Handle -> handler.GetAudit_tabless.GetAudit_tables", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAudit_tabless")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetAudit_tabless.Limit", limitParam)
	tracker.AddParam("handler.GetAudit_tabless.Offset", offsetParam)
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

	result, err := h.serviceIF.GetAudit_tables(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Audit_tables); ok {
		tracker.AddResult("handler.GetAudit_tabless.Count", len(items))
		tracker.AddResult("handler.GetAudit_tabless.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetAudit_tablesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_tables.Handle -> GetAudit_tablesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAudit_tablesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetAudit_tablesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetAudit_tablesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAudit_tablesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetAudit_tablesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetAudit_tablesByTableCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_tablesHandle -> GetAudit_tablesByTableCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetAudit_tablesByTableCode")
	defer tracker.Finish(&err)

	tablecodeParam := c.Param("tablecode")

	tablecode := tablecodeParam

	result, err := h.serviceIF.GetAudit_tablesByTableCode(c.Request().Context(), tablecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetAudit_tablesByTableCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteAudit_tablesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_tablesHandle -> DeleteAudit_tablesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteAudit_tablesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteAudit_tablesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteAudit_tablesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteAudit_tablesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteAudit_tablesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertAudit_tables(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_tablesHandle -> InsertAudit_tables", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertAudit_tables")
	defer tracker.Finish(&err)




	audit_tables := new(model.Audit_tables)
	errBind := c.Bind(audit_tables)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertAudit_tables(c.Request().Context(), audit_tables)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertAudit_tables.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateAudit_tables(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Audit_tablesHandle -> UpdateAudit_tables", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateAudit_tables")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateAudit_tables.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	audit_tables := new(model.Audit_tables)
	errBind := c.Bind(audit_tables)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateAudit_tables(c.Request().Context(), audit_tables, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateAudit_tables.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

