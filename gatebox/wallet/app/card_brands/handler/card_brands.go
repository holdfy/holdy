package card_brandsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  card_brandsSV "palm-pay/app/card_brands/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF card_brandsSV.Card_brandsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewCard_brandsHandler(service card_brandsSV.Card_brandsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("card_brands"), // <---- adicionado aqui
     }
}
func (h Handler)  GetCard_brands(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_brands.Handle -> handler.GetCard_brandss.GetCard_brands", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetCard_brandss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetCard_brandss.Limit", limitParam)
	tracker.AddParam("handler.GetCard_brandss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetCard_brands(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Card_brands); ok {
		tracker.AddResult("handler.GetCard_brandss.Count", len(items))
		tracker.AddResult("handler.GetCard_brandss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetCard_brandsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_brands.Handle -> GetCard_brandsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetCard_brandsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetCard_brandsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetCard_brandsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetCard_brandsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetCard_brandsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetCard_brandsByBrandCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_brandsHandle -> GetCard_brandsByBrandCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetCard_brandsByBrandCode")
	defer tracker.Finish(&err)

	brandcodeParam := c.Param("brandcode")

	brandcode := brandcodeParam

	result, err := h.serviceIF.GetCard_brandsByBrandCode(c.Request().Context(), brandcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetCard_brandsByBrandCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteCard_brandsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_brandsHandle -> DeleteCard_brandsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteCard_brandsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteCard_brandsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteCard_brandsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteCard_brandsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteCard_brandsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertCard_brands(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_brandsHandle -> InsertCard_brands", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertCard_brands")
	defer tracker.Finish(&err)




	card_brands := new(model.Card_brands)
	errBind := c.Bind(card_brands)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertCard_brands(c.Request().Context(), card_brands)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertCard_brands.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateCard_brands(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Card_brandsHandle -> UpdateCard_brands", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateCard_brands")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateCard_brands.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	card_brands := new(model.Card_brands)
	errBind := c.Bind(card_brands)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateCard_brands(c.Request().Context(), card_brands, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateCard_brands.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

