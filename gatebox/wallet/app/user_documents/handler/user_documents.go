package user_documentsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  user_documentsSV "palm-pay/app/user_documents/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF user_documentsSV.User_documentsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewUser_documentsHandler(service user_documentsSV.User_documentsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("user_documents"), // <---- adicionado aqui
     }
}
func (h Handler)  GetUser_documents(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("User_documents.Handle -> handler.GetUser_documentss.GetUser_documents", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_documentss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetUser_documentss.Limit", limitParam)
	tracker.AddParam("handler.GetUser_documentss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetUser_documents(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.User_documents); ok {
		tracker.AddResult("handler.GetUser_documentss.Count", len(items))
		tracker.AddResult("handler.GetUser_documentss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetUser_documentsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_documents.Handle -> GetUser_documentsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_documentsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetUser_documentsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetUser_documentsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_documentsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetUser_documentsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetUser_documentsByDocumentCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_documentsHandle -> GetUser_documentsByDocumentCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_documentsByDocumentCode")
	defer tracker.Finish(&err)

	documentcodeParam := c.Param("documentcode")

	documentcode := documentcodeParam

	result, err := h.serviceIF.GetUser_documentsByDocumentCode(c.Request().Context(), documentcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_documentsByDocumentCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteUser_documentsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_documentsHandle -> DeleteUser_documentsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteUser_documentsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteUser_documentsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteUser_documentsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteUser_documentsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteUser_documentsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertUser_documents(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_documentsHandle -> InsertUser_documents", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertUser_documents")
	defer tracker.Finish(&err)




	user_documents := new(model.User_documents)
	errBind := c.Bind(user_documents)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertUser_documents(c.Request().Context(), user_documents)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertUser_documents.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateUser_documents(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_documentsHandle -> UpdateUser_documents", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateUser_documents")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateUser_documents.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	user_documents := new(model.User_documents)
	errBind := c.Bind(user_documents)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateUser_documents(c.Request().Context(), user_documents, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateUser_documents.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

