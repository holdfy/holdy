package card_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  card_typesSV "palm-pay/app/card_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF card_typesSV.Card_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewCard_typesHandler(service card_typesSV.Card_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("card_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetCard_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_types.Handle -> handler.GetCard_typess.GetCard_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetCard_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetCard_typess.Limit", limitParam)
	tracker.AddParam("handler.GetCard_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetCard_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Card_types); ok {
		tracker.AddResult("handler.GetCard_typess.Count", len(items))
		tracker.AddResult("handler.GetCard_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetCard_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_types.Handle -> GetCard_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetCard_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetCard_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetCard_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetCard_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetCard_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetCard_typesByTypeCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_typesHandle -> GetCard_typesByTypeCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetCard_typesByTypeCode")
	defer tracker.Finish(&err)

	typecodeParam := c.Param("typecode")

	typecode := typecodeParam

	result, err := h.serviceIF.GetCard_typesByTypeCode(c.Request().Context(), typecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetCard_typesByTypeCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteCard_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_typesHandle -> DeleteCard_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteCard_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteCard_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteCard_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteCard_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteCard_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertCard_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_typesHandle -> InsertCard_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertCard_types")
	defer tracker.Finish(&err)




	card_types := new(model.Card_types)
	errBind := c.Bind(card_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertCard_types(c.Request().Context(), card_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertCard_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateCard_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_typesHandle -> UpdateCard_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateCard_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateCard_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	card_types := new(model.Card_types)
	errBind := c.Bind(card_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateCard_types(c.Request().Context(), card_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateCard_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

