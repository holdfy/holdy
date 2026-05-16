package gatewaysHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  gatewaysSV "palm-pay/app/gateways/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF gatewaysSV.GatewaysServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewGatewaysHandler(service gatewaysSV.GatewaysServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("gateways"), // <---- adicionado aqui
     }
}
func (h Handler)  GetGateways(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateways.Handle -> handler.GetGatewayss.GetGateways", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetGatewayss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetGatewayss.Limit", limitParam)
	tracker.AddParam("handler.GetGatewayss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetGateways(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Gateways); ok {
		tracker.AddResult("handler.GetGatewayss.Count", len(items))
		tracker.AddResult("handler.GetGatewayss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetGatewaysById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateways.Handle -> GetGatewaysById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetGatewaysById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetGatewaysById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetGatewaysById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetGatewaysById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetGatewaysById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetGatewaysByGatewayCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("GatewaysHandle -> GetGatewaysByGatewayCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetGatewaysByGatewayCode")
	defer tracker.Finish(&err)

	gatewaycodeParam := c.Param("gatewaycode")

	gatewaycode := gatewaycodeParam

	result, err := h.serviceIF.GetGatewaysByGatewayCode(c.Request().Context(), gatewaycode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetGatewaysByGatewayCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteGatewaysById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("GatewaysHandle -> DeleteGatewaysById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteGatewaysById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteGatewaysById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteGatewaysById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteGatewaysById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteGatewaysById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertGateways(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("GatewaysHandle -> InsertGateways", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertGateways")
	defer tracker.Finish(&err)




	gateways := new(model.Gateways)
	errBind := c.Bind(gateways)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertGateways(c.Request().Context(), gateways)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertGateways.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateGateways(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("GatewaysHandle -> UpdateGateways", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateGateways")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateGateways.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	gateways := new(model.Gateways)
	errBind := c.Bind(gateways)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateGateways(c.Request().Context(), gateways, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateGateways.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

