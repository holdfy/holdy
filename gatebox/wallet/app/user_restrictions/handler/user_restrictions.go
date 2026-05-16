package user_restrictionsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  user_restrictionsSV "palm-pay/app/user_restrictions/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF user_restrictionsSV.User_restrictionsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewUser_restrictionsHandler(service user_restrictionsSV.User_restrictionsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("user_restrictions"), // <---- adicionado aqui
     }
}
func (h Handler)  GetUser_restrictions(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("User_restrictions.Handle -> handler.GetUser_restrictionss.GetUser_restrictions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_restrictionss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetUser_restrictionss.Limit", limitParam)
	tracker.AddParam("handler.GetUser_restrictionss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetUser_restrictions(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.User_restrictions); ok {
		tracker.AddResult("handler.GetUser_restrictionss.Count", len(items))
		tracker.AddResult("handler.GetUser_restrictionss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetUser_restrictionsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_restrictions.Handle -> GetUser_restrictionsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_restrictionsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetUser_restrictionsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetUser_restrictionsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_restrictionsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetUser_restrictionsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetUser_restrictionsByRestrictionCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_restrictionsHandle -> GetUser_restrictionsByRestrictionCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_restrictionsByRestrictionCode")
	defer tracker.Finish(&err)

	restrictioncodeParam := c.Param("restrictioncode")

	restrictioncode := restrictioncodeParam

	result, err := h.serviceIF.GetUser_restrictionsByRestrictionCode(c.Request().Context(), restrictioncode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_restrictionsByRestrictionCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteUser_restrictionsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_restrictionsHandle -> DeleteUser_restrictionsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteUser_restrictionsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteUser_restrictionsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteUser_restrictionsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteUser_restrictionsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteUser_restrictionsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertUser_restrictions(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_restrictionsHandle -> InsertUser_restrictions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertUser_restrictions")
	defer tracker.Finish(&err)




	user_restrictions := new(model.User_restrictions)
	errBind := c.Bind(user_restrictions)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertUser_restrictions(c.Request().Context(), user_restrictions)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertUser_restrictions.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateUser_restrictions(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_restrictionsHandle -> UpdateUser_restrictions", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateUser_restrictions")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateUser_restrictions.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	user_restrictions := new(model.User_restrictions)
	errBind := c.Bind(user_restrictions)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateUser_restrictions(c.Request().Context(), user_restrictions, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateUser_restrictions.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

