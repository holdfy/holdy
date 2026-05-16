package session_statusSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	session_statusRepo "palm-pay/app/session_status/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Session_statusServiceIF interface {
     GetSession_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSession_statusById(ctx context.Context, id int64) (*model.Session_status, error)
     GetSession_statusByStatusCode(ctx context.Context, statuscode string) (*model.Session_status, error)
     InsertSession_status(ctx context.Context, session_status *model.Session_status) (int64, error)
     UpdateSession_status(ctx context.Context, session_status *model.Session_status, id int64) error
     DeleteSession_statusById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     session_statusRepo session_statusRepo.Session_statusRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewSession_statusService(session_statusRepo session_statusRepo.Session_statusRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         session_statusRepo: session_statusRepo,
		  observability:  observabilidade.NewServiceObservability("service.session_status"),
     }
}
func (r Resource)  GetSession_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Session_statusService -> GetSession_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSession_status.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetSession_status.offset", offset)
	tracker.AddParam("service.GetSession_status.limit", limit)



	itemsPage, err = r.session_statusRepo.GetSession_status(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Session_status); ok {
		tracker.AddResult("service.GetSession_status.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetSession_statusById(ctx context.Context, id int64) (session_status *model.Session_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Session_statusService -> GetSession_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSession_statusById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetSession_statusById.id", id)
	session_status, err = r.session_statusRepo.GetSession_statusById(ctx, id)
	if err != nil {
		return session_status, errors.New(app.MsgRepositoryError)
	}

	return session_status, nil
}
func (r Resource)  GetSession_statusByStatusCode(ctx context.Context, statuscode string) (session_status *model.Session_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Session_statusService -> GetSession_statusByStatusCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetSession_statusByStatusCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetSession_statusByStatusCode.statuscode", statuscode)
	session_status, err = r.session_statusRepo.GetSession_statusByStatusCode(ctx, statuscode)
	if err != nil {
		return session_status, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetSession_statusByStatusCode.found", session_status != nil)
	return session_status, nil
}
func (r Resource)  DeleteSession_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Session_statusService -> DeleteSession_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteSession_statusById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteSession_statusById.id",id)

	result, err = r.session_statusRepo.DeleteSession_statusById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteSession_statusById.deleted", result)
	return result, nil
}
func (r Resource)  InsertSession_status(ctx context.Context,session_status *model.Session_status) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Session_statusService -> InsertSession_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertSession_status")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertSession_status.statuscode", session_status.StatusCode)
	insertedId, err = r.session_statusRepo.InsertSession_status(ctx, session_status)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertSession_status.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateSession_status(ctx context.Context,session_status *model.Session_status, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Session_statusService -> UpdateSession_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateSession_status")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateSession_status.id", id)
	tracker.AddParam("service.UpdateSession_status.statuscode", session_status.StatusCode)

	err = r.session_statusRepo.UpdateSession_status(ctx, session_status, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateSession_status.updated", true)

	return nil
}

