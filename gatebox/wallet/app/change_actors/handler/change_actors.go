package change_actorsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  change_actorsSV "palm-pay/app/change_actors/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF change_actorsSV.Change_actorsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewChange_actorsHandler(service change_actorsSV.Change_actorsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("change_actors"), // <---- adicionado aqui
     }
}
func (h Handler)  GetChange_actors(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Change_actors.Handle -> handler.GetChange_actorss.GetChange_actors", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetChange_actorss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetChange_actorss.Limit", limitParam)
	tracker.AddParam("handler.GetChange_actorss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetChange_actors(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Change_actors); ok {
		tracker.AddResult("handler.GetChange_actorss.Count", len(items))
		tracker.AddResult("handler.GetChange_actorss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetChange_actorsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Change_actors.Handle -> GetChange_actorsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetChange_actorsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetChange_actorsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetChange_actorsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetChange_actorsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetChange_actorsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetChange_actorsByActorCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Change_actorsHandle -> GetChange_actorsByActorCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetChange_actorsByActorCode")
	defer tracker.Finish(&err)

	actorcodeParam := c.Param("actorcode")

	actorcode := actorcodeParam

	result, err := h.serviceIF.GetChange_actorsByActorCode(c.Request().Context(), actorcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetChange_actorsByActorCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteChange_actorsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Change_actorsHandle -> DeleteChange_actorsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteChange_actorsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteChange_actorsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteChange_actorsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteChange_actorsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteChange_actorsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertChange_actors(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Change_actorsHandle -> InsertChange_actors", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertChange_actors")
	defer tracker.Finish(&err)




	change_actors := new(model.Change_actors)
	errBind := c.Bind(change_actors)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertChange_actors(c.Request().Context(), change_actors)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertChange_actors.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateChange_actors(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Change_actorsHandle -> UpdateChange_actors", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateChange_actors")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateChange_actors.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	change_actors := new(model.Change_actors)
	errBind := c.Bind(change_actors)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateChange_actors(c.Request().Context(), change_actors, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateChange_actors.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

