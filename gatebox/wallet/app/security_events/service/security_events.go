package security_eventsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	security_eventsRepo "palm-pay/app/security_events/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Security_eventsServiceIF interface {
     GetSecurity_events(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSecurity_eventsById(ctx context.Context, id int64) (*model.Security_events, error)
     GetSecurity_eventsBySecurityEventCode(ctx context.Context, securityeventcode string) (*model.Security_events, error)
     InsertSecurity_events(ctx context.Context, security_events *model.Security_events) (int64, error)
     UpdateSecurity_events(ctx context.Context, security_events *model.Security_events, id int64) error
     DeleteSecurity_eventsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     security_eventsRepo security_eventsRepo.Security_eventsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewSecurity_eventsService(security_eventsRepo security_eventsRepo.Security_eventsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         security_eventsRepo: security_eventsRepo,
		  observability:  observabilidade.NewServiceObservability("service.security_events"),
     }
}
func (r Resource)  GetSecurity_events(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_eventsService -> GetSecurity_events", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSecurity_events.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetSecurity_events.offset", offset)
	tracker.AddParam("service.GetSecurity_events.limit", limit)



	itemsPage, err = r.security_eventsRepo.GetSecurity_events(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Security_events); ok {
		tracker.AddResult("service.GetSecurity_events.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetSecurity_eventsById(ctx context.Context, id int64) (security_events *model.Security_events, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_eventsService -> GetSecurity_eventsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSecurity_eventsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetSecurity_eventsById.id", id)
	security_events, err = r.security_eventsRepo.GetSecurity_eventsById(ctx, id)
	if err != nil {
		return security_events, errors.New(app.MsgRepositoryError)
	}

	return security_events, nil
}
func (r Resource)  GetSecurity_eventsBySecurityEventCode(ctx context.Context, securityeventcode string) (security_events *model.Security_events, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_eventsService -> GetSecurity_eventsBySecurityEventCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetSecurity_eventsBySecurityEventCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetSecurity_eventsBySecurityEventCode.securityeventcode", securityeventcode)
	security_events, err = r.security_eventsRepo.GetSecurity_eventsBySecurityEventCode(ctx, securityeventcode)
	if err != nil {
		return security_events, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetSecurity_eventsBySecurityEventCode.found", security_events != nil)
	return security_events, nil
}
func (r Resource)  DeleteSecurity_eventsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_eventsService -> DeleteSecurity_eventsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteSecurity_eventsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteSecurity_eventsById.id",id)

	result, err = r.security_eventsRepo.DeleteSecurity_eventsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteSecurity_eventsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertSecurity_events(ctx context.Context,security_events *model.Security_events) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_eventsService -> InsertSecurity_events", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertSecurity_events")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertSecurity_events.securityeventcode", security_events.SecurityEventCode)
	insertedId, err = r.security_eventsRepo.InsertSecurity_events(ctx, security_events)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertSecurity_events.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateSecurity_events(ctx context.Context,security_events *model.Security_events, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_eventsService -> UpdateSecurity_events", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateSecurity_events")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateSecurity_events.id", id)
	tracker.AddParam("service.UpdateSecurity_events.securityeventcode", security_events.SecurityEventCode)

	err = r.security_eventsRepo.UpdateSecurity_events(ctx, security_events, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateSecurity_events.updated", true)

	return nil
}

