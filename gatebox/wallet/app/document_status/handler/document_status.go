package document_statusHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  document_statusSV "palm-pay/app/document_status/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF document_statusSV.Document_statusServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewDocument_statusHandler(service document_statusSV.Document_statusServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("document_status"), // <---- adicionado aqui
     }
}
func (h Handler)  GetDocument_status(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_status.Handle -> handler.GetDocument_statuss.GetDocument_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetDocument_statuss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetDocument_statuss.Limit", limitParam)
	tracker.AddParam("handler.GetDocument_statuss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetDocument_status(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Document_status); ok {
		tracker.AddResult("handler.GetDocument_statuss.Count", len(items))
		tracker.AddResult("handler.GetDocument_statuss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetDocument_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_status.Handle -> GetDocument_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetDocument_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetDocument_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetDocument_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetDocument_statusById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetDocument_statusById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetDocument_statusByStatusCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_statusHandle -> GetDocument_statusByStatusCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetDocument_statusByStatusCode")
	defer tracker.Finish(&err)

	statuscodeParam := c.Param("statuscode")

	statuscode := statuscodeParam

	result, err := h.serviceIF.GetDocument_statusByStatusCode(c.Request().Context(), statuscode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetDocument_statusByStatusCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteDocument_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_statusHandle -> DeleteDocument_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteDocument_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteDocument_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteDocument_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteDocument_statusById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteDocument_statusById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertDocument_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_statusHandle -> InsertDocument_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertDocument_status")
	defer tracker.Finish(&err)




	document_status := new(model.Document_status)
	errBind := c.Bind(document_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertDocument_status(c.Request().Context(), document_status)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertDocument_status.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateDocument_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_statusHandle -> UpdateDocument_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateDocument_status")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateDocument_status.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	document_status := new(model.Document_status)
	errBind := c.Bind(document_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateDocument_status(c.Request().Context(), document_status, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateDocument_status.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

