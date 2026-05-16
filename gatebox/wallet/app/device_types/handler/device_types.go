package device_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  device_typesSV "palm-pay/app/device_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF device_typesSV.Device_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewDevice_typesHandler(service device_typesSV.Device_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("device_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetDevice_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Device_types.Handle -> handler.GetDevice_typess.GetDevice_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetDevice_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetDevice_typess.Limit", limitParam)
	tracker.AddParam("handler.GetDevice_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetDevice_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Device_types); ok {
		tracker.AddResult("handler.GetDevice_typess.Count", len(items))
		tracker.AddResult("handler.GetDevice_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetDevice_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Device_types.Handle -> GetDevice_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetDevice_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetDevice_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetDevice_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetDevice_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetDevice_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetDevice_typesByTypeCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Device_typesHandle -> GetDevice_typesByTypeCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetDevice_typesByTypeCode")
	defer tracker.Finish(&err)

	typecodeParam := c.Param("typecode")

	typecode := typecodeParam

	result, err := h.serviceIF.GetDevice_typesByTypeCode(c.Request().Context(), typecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetDevice_typesByTypeCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteDevice_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Device_typesHandle -> DeleteDevice_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteDevice_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteDevice_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteDevice_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteDevice_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteDevice_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertDevice_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Device_typesHandle -> InsertDevice_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertDevice_types")
	defer tracker.Finish(&err)




	device_types := new(model.Device_types)
	errBind := c.Bind(device_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertDevice_types(c.Request().Context(), device_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertDevice_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateDevice_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Device_typesHandle -> UpdateDevice_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateDevice_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateDevice_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	device_types := new(model.Device_types)
	errBind := c.Bind(device_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateDevice_types(c.Request().Context(), device_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateDevice_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

