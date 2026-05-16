package document_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  document_typesSV "palm-pay/app/document_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF document_typesSV.Document_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewDocument_typesHandler(service document_typesSV.Document_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("document_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetDocument_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_types.Handle -> handler.GetDocument_typess.GetDocument_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetDocument_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetDocument_typess.Limit", limitParam)
	tracker.AddParam("handler.GetDocument_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetDocument_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Document_types); ok {
		tracker.AddResult("handler.GetDocument_typess.Count", len(items))
		tracker.AddResult("handler.GetDocument_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetDocument_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_types.Handle -> GetDocument_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetDocument_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetDocument_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetDocument_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetDocument_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetDocument_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetDocument_typesByTypeCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_typesHandle -> GetDocument_typesByTypeCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetDocument_typesByTypeCode")
	defer tracker.Finish(&err)

	typecodeParam := c.Param("typecode")

	typecode := typecodeParam

	result, err := h.serviceIF.GetDocument_typesByTypeCode(c.Request().Context(), typecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetDocument_typesByTypeCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteDocument_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_typesHandle -> DeleteDocument_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteDocument_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteDocument_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteDocument_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteDocument_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteDocument_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertDocument_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_typesHandle -> InsertDocument_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertDocument_types")
	defer tracker.Finish(&err)




	document_types := new(model.Document_types)
	errBind := c.Bind(document_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertDocument_types(c.Request().Context(), document_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertDocument_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateDocument_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Document_typesHandle -> UpdateDocument_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateDocument_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateDocument_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	document_types := new(model.Document_types)
	errBind := c.Bind(document_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateDocument_types(c.Request().Context(), document_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateDocument_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

