package user_sessionsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  user_sessionsSV "palm-pay/app/user_sessions/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF user_sessionsSV.User_sessionsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewUser_sessionsHandler(service user_sessionsSV.User_sessionsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("user_sessions"), // <---- adicionado aqui
     }
}
func (h Handler)  GetUser_sessions(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("User_sessions.Handle -> handler.GetUser_sessionss.GetUser_sessions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_sessionss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetUser_sessionss.Limit", limitParam)
	tracker.AddParam("handler.GetUser_sessionss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetUser_sessions(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.User_sessions); ok {
		tracker.AddResult("handler.GetUser_sessionss.Count", len(items))
		tracker.AddResult("handler.GetUser_sessionss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetUser_sessionsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_sessions.Handle -> GetUser_sessionsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_sessionsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetUser_sessionsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetUser_sessionsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_sessionsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetUser_sessionsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetUser_sessionsBySessionCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_sessionsHandle -> GetUser_sessionsBySessionCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_sessionsBySessionCode")
	defer tracker.Finish(&err)

	sessioncodeParam := c.Param("sessioncode")

	sessioncode := sessioncodeParam

	result, err := h.serviceIF.GetUser_sessionsBySessionCode(c.Request().Context(), sessioncode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_sessionsBySessionCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteUser_sessionsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_sessionsHandle -> DeleteUser_sessionsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteUser_sessionsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteUser_sessionsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteUser_sessionsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteUser_sessionsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteUser_sessionsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertUser_sessions(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_sessionsHandle -> InsertUser_sessions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertUser_sessions")
	defer tracker.Finish(&err)




	user_sessions := new(model.User_sessions)
	errBind := c.Bind(user_sessions)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertUser_sessions(c.Request().Context(), user_sessions)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertUser_sessions.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateUser_sessions(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_sessionsHandle -> UpdateUser_sessions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateUser_sessions")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateUser_sessions.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	user_sessions := new(model.User_sessions)
	errBind := c.Bind(user_sessions)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateUser_sessions(c.Request().Context(), user_sessions, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateUser_sessions.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

