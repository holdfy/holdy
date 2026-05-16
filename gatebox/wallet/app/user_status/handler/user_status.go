package user_statusHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  user_statusSV "palm-pay/app/user_status/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF user_statusSV.User_statusServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewUser_statusHandler(service user_statusSV.User_statusServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("user_status"), // <---- adicionado aqui
     }
}
func (h Handler)  GetUser_status(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("User_status.Handle -> handler.GetUser_statuss.GetUser_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_statuss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetUser_statuss.Limit", limitParam)
	tracker.AddParam("handler.GetUser_statuss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetUser_status(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.User_status); ok {
		tracker.AddResult("handler.GetUser_statuss.Count", len(items))
		tracker.AddResult("handler.GetUser_statuss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetUser_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_status.Handle -> GetUser_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetUser_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetUser_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_statusById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetUser_statusById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetUser_statusByStatusCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_statusHandle -> GetUser_statusByStatusCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetUser_statusByStatusCode")
	defer tracker.Finish(&err)

	statuscodeParam := c.Param("statuscode")

	statuscode := statuscodeParam

	result, err := h.serviceIF.GetUser_statusByStatusCode(c.Request().Context(), statuscode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetUser_statusByStatusCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteUser_statusById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_statusHandle -> DeleteUser_statusById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteUser_statusById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteUser_statusById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteUser_statusById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteUser_statusById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteUser_statusById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertUser_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_statusHandle -> InsertUser_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertUser_status")
	defer tracker.Finish(&err)




	user_status := new(model.User_status)
	errBind := c.Bind(user_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertUser_status(c.Request().Context(), user_status)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertUser_status.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateUser_status(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("User_statusHandle -> UpdateUser_status", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateUser_status")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateUser_status.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	user_status := new(model.User_status)
	errBind := c.Bind(user_status)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateUser_status(c.Request().Context(), user_status, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateUser_status.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

