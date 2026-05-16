package user_cardsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  user_cardsSV "palm-pay/app/user_cards/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF user_cardsSV.User_cardsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewUser_cardsHandler(service user_cardsSV.User_cardsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("user_cards"), // <---- adicionado aqui
     }
}
func (h Handler)  GetUser_cards(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("User_cards.Handle -> handler.GetUser_cardss.GetUser_cards", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_cardss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetUser_cardss.Limit", limitParam)
	tracker.AddParam("handler.GetUser_cardss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetUser_cards(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.User_cards); ok {
		tracker.AddResult("handler.GetUser_cardss.Count", len(items))
		tracker.AddResult("handler.GetUser_cardss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetUser_cardsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_cards.Handle -> GetUser_cardsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_cardsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetUser_cardsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetUser_cardsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_cardsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetUser_cardsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetUser_cardsByCardCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_cardsHandle -> GetUser_cardsByCardCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_cardsByCardCode")
	defer tracker.Finish(&err)

	cardcodeParam := c.Param("cardcode")

	cardcode := cardcodeParam

	result, err := h.serviceIF.GetUser_cardsByCardCode(c.Request().Context(), cardcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_cardsByCardCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteUser_cardsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_cardsHandle -> DeleteUser_cardsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteUser_cardsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteUser_cardsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteUser_cardsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteUser_cardsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteUser_cardsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertUser_cards(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_cardsHandle -> InsertUser_cards", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertUser_cards")
	defer tracker.Finish(&err)




	user_cards := new(model.User_cards)
	errBind := c.Bind(user_cards)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertUser_cards(c.Request().Context(), user_cards)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertUser_cards.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateUser_cards(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_cardsHandle -> UpdateUser_cards", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateUser_cards")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateUser_cards.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	user_cards := new(model.User_cards)
	errBind := c.Bind(user_cards)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateUser_cards(c.Request().Context(), user_cards, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateUser_cards.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

