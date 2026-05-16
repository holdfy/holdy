package restriction_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  restriction_typesSV "palm-pay/app/restriction_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF restriction_typesSV.Restriction_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewRestriction_typesHandler(service restriction_typesSV.Restriction_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("restriction_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetRestriction_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Restriction_types.Handle -> handler.GetRestriction_typess.GetRestriction_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetRestriction_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetRestriction_typess.Limit", limitParam)
	tracker.AddParam("handler.GetRestriction_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetRestriction_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Restriction_types); ok {
		tracker.AddResult("handler.GetRestriction_typess.Count", len(items))
		tracker.AddResult("handler.GetRestriction_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetRestriction_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Restriction_types.Handle -> GetRestriction_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetRestriction_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetRestriction_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetRestriction_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetRestriction_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetRestriction_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetRestriction_typesByTypeCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Restriction_typesHandle -> GetRestriction_typesByTypeCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetRestriction_typesByTypeCode")
	defer tracker.Finish(&err)

	typecodeParam := c.Param("typecode")

	typecode := typecodeParam

	result, err := h.serviceIF.GetRestriction_typesByTypeCode(c.Request().Context(), typecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetRestriction_typesByTypeCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteRestriction_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Restriction_typesHandle -> DeleteRestriction_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteRestriction_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteRestriction_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteRestriction_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteRestriction_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteRestriction_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertRestriction_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Restriction_typesHandle -> InsertRestriction_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertRestriction_types")
	defer tracker.Finish(&err)




	restriction_types := new(model.Restriction_types)
	errBind := c.Bind(restriction_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertRestriction_types(c.Request().Context(), restriction_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertRestriction_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateRestriction_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Restriction_typesHandle -> UpdateRestriction_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateRestriction_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateRestriction_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	restriction_types := new(model.Restriction_types)
	errBind := c.Bind(restriction_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateRestriction_types(c.Request().Context(), restriction_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateRestriction_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

