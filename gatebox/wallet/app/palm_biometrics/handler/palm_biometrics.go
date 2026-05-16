package palm_biometricsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  palm_biometricsSV "palm-pay/app/palm_biometrics/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF palm_biometricsSV.Palm_biometricsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewPalm_biometricsHandler(service palm_biometricsSV.Palm_biometricsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("palm_biometrics"), // <---- adicionado aqui
     }
}
func (h Handler)  GetPalm_biometrics(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Palm_biometrics.Handle -> handler.GetPalm_biometricss.GetPalm_biometrics", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetPalm_biometricss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetPalm_biometricss.Limit", limitParam)
	tracker.AddParam("handler.GetPalm_biometricss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetPalm_biometrics(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Palm_biometrics); ok {
		tracker.AddResult("handler.GetPalm_biometricss.Count", len(items))
		tracker.AddResult("handler.GetPalm_biometricss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetPalm_biometricsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Palm_biometrics.Handle -> GetPalm_biometricsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetPalm_biometricsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetPalm_biometricsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetPalm_biometricsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetPalm_biometricsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetPalm_biometricsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetPalm_biometricsByBiometricCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Palm_biometricsHandle -> GetPalm_biometricsByBiometricCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetPalm_biometricsByBiometricCode")
	defer tracker.Finish(&err)

	biometriccodeParam := c.Param("biometriccode")

	biometriccode := biometriccodeParam

	result, err := h.serviceIF.GetPalm_biometricsByBiometricCode(c.Request().Context(), biometriccode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetPalm_biometricsByBiometricCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeletePalm_biometricsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Palm_biometricsHandle -> DeletePalm_biometricsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeletePalm_biometricsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeletePalm_biometricsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeletePalm_biometricsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeletePalm_biometricsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeletePalm_biometricsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertPalm_biometrics(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Palm_biometricsHandle -> InsertPalm_biometrics", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertPalm_biometrics")
	defer tracker.Finish(&err)




	palm_biometrics := new(model.Palm_biometrics)
	errBind := c.Bind(palm_biometrics)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertPalm_biometrics(c.Request().Context(), palm_biometrics)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertPalm_biometrics.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdatePalm_biometrics(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Palm_biometricsHandle -> UpdatePalm_biometrics", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdatePalm_biometrics")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdatePalm_biometrics.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	palm_biometrics := new(model.Palm_biometrics)
	errBind := c.Bind(palm_biometrics)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdatePalm_biometrics(c.Request().Context(), palm_biometrics, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdatePalm_biometrics.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

