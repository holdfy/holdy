package applicationsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  applicationsSV "palm-pay/app/applications/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF applicationsSV.ApplicationsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewApplicationsHandler(service applicationsSV.ApplicationsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("applications"), // <---- adicionado aqui
     }
}
func (h Handler)  GetApplications(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Applications.Handle -> handler.GetApplicationss.GetApplications", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetApplicationss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetApplicationss.Limit", limitParam)
	tracker.AddParam("handler.GetApplicationss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetApplications(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Applications); ok {
		tracker.AddResult("handler.GetApplicationss.Count", len(items))
		tracker.AddResult("handler.GetApplicationss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetApplicationsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Applications.Handle -> GetApplicationsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetApplicationsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetApplicationsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetApplicationsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetApplicationsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetApplicationsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetApplicationsByAppCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("ApplicationsHandle -> GetApplicationsByAppCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetApplicationsByAppCode")
	defer tracker.Finish(&err)

	appcodeParam := c.Param("appcode")

	appcode := appcodeParam

	result, err := h.serviceIF.GetApplicationsByAppCode(c.Request().Context(), appcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetApplicationsByAppCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteApplicationsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("ApplicationsHandle -> DeleteApplicationsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteApplicationsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteApplicationsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteApplicationsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteApplicationsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteApplicationsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertApplications(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("ApplicationsHandle -> InsertApplications", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertApplications")
	defer tracker.Finish(&err)




	applications := new(model.Applications)
	errBind := c.Bind(applications)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertApplications(c.Request().Context(), applications)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertApplications.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateApplications(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("ApplicationsHandle -> UpdateApplications", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateApplications")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateApplications.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	applications := new(model.Applications)
	errBind := c.Bind(applications)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateApplications(c.Request().Context(), applications, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateApplications.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

