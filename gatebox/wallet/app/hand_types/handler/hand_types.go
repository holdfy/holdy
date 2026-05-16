package hand_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  hand_typesSV "palm-pay/app/hand_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF hand_typesSV.Hand_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewHand_typesHandler(service hand_typesSV.Hand_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("hand_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetHand_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Hand_types.Handle -> handler.GetHand_typess.GetHand_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetHand_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetHand_typess.Limit", limitParam)
	tracker.AddParam("handler.GetHand_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetHand_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Hand_types); ok {
		tracker.AddResult("handler.GetHand_typess.Count", len(items))
		tracker.AddResult("handler.GetHand_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetHand_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Hand_types.Handle -> GetHand_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetHand_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetHand_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetHand_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetHand_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetHand_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetHand_typesByTypeCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Hand_typesHandle -> GetHand_typesByTypeCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetHand_typesByTypeCode")
	defer tracker.Finish(&err)

	typecodeParam := c.Param("typecode")

	typecode := typecodeParam

	result, err := h.serviceIF.GetHand_typesByTypeCode(c.Request().Context(), typecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetHand_typesByTypeCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteHand_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Hand_typesHandle -> DeleteHand_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteHand_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteHand_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteHand_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteHand_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteHand_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertHand_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Hand_typesHandle -> InsertHand_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertHand_types")
	defer tracker.Finish(&err)




	hand_types := new(model.Hand_types)
	errBind := c.Bind(hand_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertHand_types(c.Request().Context(), hand_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertHand_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateHand_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Hand_typesHandle -> UpdateHand_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateHand_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateHand_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	hand_types := new(model.Hand_types)
	errBind := c.Bind(hand_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateHand_types(c.Request().Context(), hand_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateHand_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

