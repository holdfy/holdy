package biometric_attemptsHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  biometric_attemptsSV "palm-pay/app/biometric_attempts/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF biometric_attemptsSV.Biometric_attemptsServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewBiometric_attemptsHandler(service biometric_attemptsSV.Biometric_attemptsServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("biometric_attempts"), // <---- adicionado aqui
     }
}
func (h Handler)  GetBiometric_attempts(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Biometric_attempts.Handle -> handler.GetBiometric_attemptss.GetBiometric_attempts", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetBiometric_attemptss")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetBiometric_attemptss.Limit", limitParam)
	tracker.AddParam("handler.GetBiometric_attemptss.Offset", offsetParam)
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

	result, err := h.serviceIF.GetBiometric_attempts(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Biometric_attempts); ok {
		tracker.AddResult("handler.GetBiometric_attemptss.Count", len(items))
		tracker.AddResult("handler.GetBiometric_attemptss.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetBiometric_attemptsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Biometric_attempts.Handle -> GetBiometric_attemptsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetBiometric_attemptsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetBiometric_attemptsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetBiometric_attemptsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetBiometric_attemptsById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetBiometric_attemptsById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetBiometric_attemptsByAttemptCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Biometric_attemptsHandle -> GetBiometric_attemptsByAttemptCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetBiometric_attemptsByAttemptCode")
	defer tracker.Finish(&err)

	attemptcodeParam := c.Param("attemptcode")

	attemptcode := attemptcodeParam

	result, err := h.serviceIF.GetBiometric_attemptsByAttemptCode(c.Request().Context(), attemptcode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetBiometric_attemptsByAttemptCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteBiometric_attemptsById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Biometric_attemptsHandle -> DeleteBiometric_attemptsById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteBiometric_attemptsById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteBiometric_attemptsById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteBiometric_attemptsById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteBiometric_attemptsById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteBiometric_attemptsById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertBiometric_attempts(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Biometric_attemptsHandle -> InsertBiometric_attempts", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertBiometric_attempts")
	defer tracker.Finish(&err)




	biometric_attempts := new(model.Biometric_attempts)
	errBind := c.Bind(biometric_attempts)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertBiometric_attempts(c.Request().Context(), biometric_attempts)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertBiometric_attempts.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateBiometric_attempts(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Biometric_attemptsHandle -> UpdateBiometric_attempts", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateBiometric_attempts")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateBiometric_attempts.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	biometric_attempts := new(model.Biometric_attempts)
	errBind := c.Bind(biometric_attempts)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateBiometric_attempts(c.Request().Context(), biometric_attempts, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateBiometric_attempts.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

