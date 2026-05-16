package signature_methodsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  signature_methodsSV "palm-pay/app/signature_methods/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF signature_methodsSV.Signature_methodsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewSignature_methodsHandler(service signature_methodsSV.Signature_methodsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("signature_methods"), // <---- adicionado aqui
     }
}
func (h Handler)  GetSignature_methods(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Signature_methods.Handle -> handler.GetSignature_methodss.GetSignature_methods", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSignature_methodss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetSignature_methodss.Limit", limitParam)
	tracker.AddParam("handler.GetSignature_methodss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetSignature_methods(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Signature_methods); ok {
		tracker.AddResult("handler.GetSignature_methodss.Count", len(items))
		tracker.AddResult("handler.GetSignature_methodss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetSignature_methodsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Signature_methods.Handle -> GetSignature_methodsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSignature_methodsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetSignature_methodsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetSignature_methodsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetSignature_methodsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetSignature_methodsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetSignature_methodsByMethodCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Signature_methodsHandle -> GetSignature_methodsByMethodCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetSignature_methodsByMethodCode")
	defer tracker.Finish(&err)

	methodcodeParam := c.Param("methodcode")

	methodcode := methodcodeParam

	result, err := h.serviceIF.GetSignature_methodsByMethodCode(c.Request().Context(), methodcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetSignature_methodsByMethodCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteSignature_methodsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Signature_methodsHandle -> DeleteSignature_methodsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteSignature_methodsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteSignature_methodsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteSignature_methodsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteSignature_methodsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteSignature_methodsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertSignature_methods(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Signature_methodsHandle -> InsertSignature_methods", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertSignature_methods")
	defer tracker.Finish(&err)




	signature_methods := new(model.Signature_methods)
	errBind := c.Bind(signature_methods)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertSignature_methods(c.Request().Context(), signature_methods)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertSignature_methods.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateSignature_methods(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Signature_methodsHandle -> UpdateSignature_methods", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateSignature_methods")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateSignature_methods.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	signature_methods := new(model.Signature_methods)
	errBind := c.Bind(signature_methods)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateSignature_methods(c.Request().Context(), signature_methods, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateSignature_methods.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

