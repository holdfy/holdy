package transaction_typesHandler

import (
  "net/http"
  "strconv"
  "time"
  app "palm-pay/app"
  transaction_typesSV "palm-pay/app/transaction_types/service"
  "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
    "github.com/labstack/echo/v4"
     "palm-pay/utils/observabilidade" // <---- adicionado aqui
)
 type Handler struct {
     serviceIF transaction_typesSV.Transaction_typesServiceIF
     log     logger.Logger
	  observability *observabilidade.HandlerObservability // <---- adicionado aqui: Um único observador
}
 func NewTransaction_typesHandler(service transaction_typesSV.Transaction_typesServiceIF, log logger.Logger) *Handler{
    return &Handler{
         log:     log,
         serviceIF: service,
		  observability: observabilidade.NewHandlerObservability("transaction_types"), // <---- adicionado aqui
     }
}
func (h Handler)  GetTransaction_types(c echo.Context) error {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_types.Handle -> handler.GetTransaction_typess.GetTransaction_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransaction_typess")


	limitParam := c.QueryParam("limit")
	offsetParam := c.QueryParam("offset")

	tracker.AddParam("handler.GetTransaction_typess.Limit", limitParam)
	tracker.AddParam("handler.GetTransaction_typess.Offset", offsetParam)
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

	result, err := h.serviceIF.GetTransaction_types(c.Request().Context(), offset, limit)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	if items, ok := result.Items.([]model.Transaction_types); ok {
		tracker.AddResult("handler.GetTransaction_typess.Count", len(items))
		tracker.AddResult("handler.GetTransaction_typess.Total", result.Total)
	}

	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  GetTransaction_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_types.Handle -> GetTransaction_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransaction_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
	tracker.AddParam("handler.GetTransaction_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	result, err := h.serviceIF.GetTransaction_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetTransaction_typesById.found", result != nil)
	if result != nil {
		tracker.AddResult("handler.GetTransaction_typesById.notFound", result.ID)

	}
	return c.JSON(http.StatusOK, &result)
}
func (h Handler)  GetTransaction_typesByTypeCode(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_typesHandle -> GetTransaction_typesByTypeCode", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.GetTransaction_typesByTypeCode")
	defer tracker.Finish(&err)

	typecodeParam := c.Param("typecode")

	typecode := typecodeParam

	result, err := h.serviceIF.GetTransaction_typesByTypeCode(c.Request().Context(), typecode)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	tracker.AddResult("handler.GetTransaction_typesByTypeCode.found", result != nil)
	_ = c.JSON(http.StatusOK, &result)
	return nil
}
func (h Handler)  DeleteTransaction_typesById(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_typesHandle -> DeleteTransaction_typesById", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.DeleteTransaction_typesById")
	defer tracker.Finish(&err)

	idParam := c.Param("id")
   tracker.AddParam("handler.DeleteTransaction_typesById.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	deleted, err := h.serviceIF.DeleteTransaction_typesById(c.Request().Context(), id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	if !deleted {
		tracker.AddResult("handler.DeleteTransaction_typesById.not_found", true)
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgFail,
		})

		return nil
	}

	tracker.AddResult("handler.DeleteTransaction_typesById.deleted", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}
func (h Handler)  InsertTransaction_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_typesHandle -> InsertTransaction_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.InsertTransaction_types")
	defer tracker.Finish(&err)




	transaction_types := new(model.Transaction_types)
	errBind := c.Bind(transaction_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	insertedId, err := h.serviceIF.InsertTransaction_types(c.Request().Context(), transaction_types)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}

	tracker.AddResult("handler.InsertTransaction_types.inserted_id", insertedId)


	_ = c.JSON(http.StatusCreated, map[string]interface{}{
		"id": insertedId,
	})
	return nil
}
func (h Handler)  UpdateTransaction_types(c echo.Context) (err error) {
	startedAt := time.Now()
	defer h.log.Chronometer("Transaction_typesHandle -> UpdateTransaction_types", &startedAt)

	tracker := h.observability.Track(c.Request().Context(), "handler.UpdateTransaction_types")
	defer tracker.Finish(&err)

    idParam := c.Param("id")
	 tracker.AddParam("handler.UpdateTransaction_types.id", idParam)

	id, err := strconv.ParseInt(idParam, 10, 64)
	if err != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}

	transaction_types := new(model.Transaction_types)
	errBind := c.Bind(transaction_types)

	if errBind != nil {
		_ = c.JSON(http.StatusBadRequest, map[string]string{
			app.Message: app.MsgBadRequest,
		})
		return nil
	}


	err = h.serviceIF.UpdateTransaction_types(c.Request().Context(), transaction_types, id)
	if err != nil {
		_ = c.JSON(http.StatusInternalServerError, map[string]string{
			app.Message: app.MsgInternalError,
		})
		return nil
	}


	tracker.AddResult("handler.UpdateTransaction_types.update+", true)
	_ = c.NoContent(http.StatusNoContent)
	return nil
}

