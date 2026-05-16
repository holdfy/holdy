package gateway_status_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  gateway_status_typesSV "palm-pay/app/gateway_status_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF gateway_status_typesSV.Gateway_status_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewGateway_status_typesHandler(service gateway_status_typesSV.Gateway_status_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("gateway_status_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetGateway_status_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_status_types.Handle -> handler.GetGateway_status_typess.GetGateway_status_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetGateway_status_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetGateway_status_typess.Limit", limitParam)
	tracker.AddParam("handler.GetGateway_status_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetGateway_status_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Gateway_status_types); ok {
		tracker.AddResult("handler.GetGateway_status_typess.Count", len(items))
		tracker.AddResult("handler.GetGateway_status_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetGateway_status_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_status_types.Handle -> GetGateway_status_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetGateway_status_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetGateway_status_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetGateway_status_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetGateway_status_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetGateway_status_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetGateway_status_typesByStatusCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_status_typesHandle -> GetGateway_status_typesByStatusCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetGateway_status_typesByStatusCode")
	defer tracker.Finish(&err)

	statuscodeParam := c.Param("statuscode")

	statuscode := statuscodeParam

	result, err := h.serviceIF.GetGateway_status_typesByStatusCode(c.Request().Context(), statuscode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetGateway_status_typesByStatusCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteGateway_status_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_status_typesHandle -> DeleteGateway_status_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteGateway_status_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteGateway_status_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteGateway_status_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteGateway_status_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteGateway_status_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertGateway_status_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_status_typesHandle -> InsertGateway_status_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertGateway_status_types")
	defer tracker.Finish(&err)




	gateway_status_types := new(model.Gateway_status_types)
	errBind := c.Bind(gateway_status_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertGateway_status_types(c.Request().Context(), gateway_status_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertGateway_status_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateGateway_status_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Gateway_status_typesHandle -> UpdateGateway_status_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateGateway_status_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateGateway_status_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	gateway_status_types := new(model.Gateway_status_types)
	errBind := c.Bind(gateway_status_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateGateway_status_types(c.Request().Context(), gateway_status_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateGateway_status_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

