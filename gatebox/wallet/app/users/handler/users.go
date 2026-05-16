package usersHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  usersSV "palm-pay/app/users/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF usersSV.UsersServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewUsersHandler(service usersSV.UsersServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("users"), // <---- adicionado aqui
     }
}
func (h Handler)  GetUsers(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Users.Handle -> handler.GetUserss.GetUsers", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUserss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetUserss.Limit", limitParam)
	tracker.AddParam("handler.GetUserss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetUsers(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Users); ok {
		tracker.AddResult("handler.GetUserss.Count", len(items))
		tracker.AddResult("handler.GetUserss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetUsersById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Users.Handle -> GetUsersById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUsersById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetUsersById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetUsersById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUsersById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetUsersById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetUsersByUserCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("UsersHandle -> GetUsersByUserCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUsersByUserCode")
	defer tracker.Finish(&err)

	usercodeParam := c.Param("usercode")

	usercode := usercodeParam

	result, err := h.serviceIF.GetUsersByUserCode(c.Request().Context(), usercode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUsersByUserCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteUsersById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("UsersHandle -> DeleteUsersById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteUsersById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteUsersById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteUsersById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteUsersById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteUsersById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertUsers(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("UsersHandle -> InsertUsers", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertUsers")
	defer tracker.Finish(&err)




	users := new(model.Users)
	errBind := c.Bind(users)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertUsers(c.Request().Context(), users)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertUsers.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateUsers(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("UsersHandle -> UpdateUsers", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateUsers")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateUsers.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	users := new(model.Users)
	errBind := c.Bind(users)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateUsers(c.Request().Context(), users, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateUsers.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

