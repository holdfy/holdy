package payment_methodsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  payment_methodsSV "palm-pay/app/payment_methods/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF payment_methodsSV.Payment_methodsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewPayment_methodsHandler(service payment_methodsSV.Payment_methodsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("payment_methods"), // <---- adicionado aqui
     }
}
func (h Handler)  GetPayment_methods(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Payment_methods.Handle -> handler.GetPayment_methodss.GetPayment_methods", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetPayment_methodss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetPayment_methodss.Limit", limitParam)
	tracker.AddParam("handler.GetPayment_methodss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetPayment_methods(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Payment_methods); ok {
		tracker.AddResult("handler.GetPayment_methodss.Count", len(items))
		tracker.AddResult("handler.GetPayment_methodss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetPayment_methodsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Payment_methods.Handle -> GetPayment_methodsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetPayment_methodsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetPayment_methodsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetPayment_methodsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetPayment_methodsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetPayment_methodsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetPayment_methodsByMethodCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Payment_methodsHandle -> GetPayment_methodsByMethodCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetPayment_methodsByMethodCode")
	defer tracker.Finish(&err)

	methodcodeParam := c.Param("methodcode")

	methodcode := methodcodeParam

	result, err := h.serviceIF.GetPayment_methodsByMethodCode(c.Request().Context(), methodcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetPayment_methodsByMethodCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeletePayment_methodsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Payment_methodsHandle -> DeletePayment_methodsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeletePayment_methodsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeletePayment_methodsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeletePayment_methodsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeletePayment_methodsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeletePayment_methodsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertPayment_methods(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Payment_methodsHandle -> InsertPayment_methods", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertPayment_methods")
	defer tracker.Finish(&err)




	payment_methods := new(model.Payment_methods)
	errBind := c.Bind(payment_methods)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertPayment_methods(c.Request().Context(), payment_methods)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertPayment_methods.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdatePayment_methods(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Payment_methodsHandle -> UpdatePayment_methods", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdatePayment_methods")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdatePayment_methods.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	payment_methods := new(model.Payment_methods)
	errBind := c.Bind(payment_methods)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdatePayment_methods(c.Request().Context(), payment_methods, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdatePayment_methods.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

