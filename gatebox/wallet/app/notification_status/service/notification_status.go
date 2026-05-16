package notification_statusSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	notification_statusRepo "palm-pay/app/notification_status/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Notification_statusServiceIF interface {
     GetNotification_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetNotification_statusById(ctx context.Context, id int64) (*model.Notification_status, error)
     GetNotification_statusByStatusCode(ctx context.Context, statuscode string) (*model.Notification_status, error)
     InsertNotification_status(ctx context.Context, notification_status *model.Notification_status) (int64, error)
     UpdateNotification_status(ctx context.Context, notification_status *model.Notification_status, id int64) error
     DeleteNotification_statusById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     notification_statusRepo notification_statusRepo.Notification_statusRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewNotification_statusService(notification_statusRepo notification_statusRepo.Notification_statusRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         notification_statusRepo: notification_statusRepo,
		  observability:  observabilidade.NewServiceObservability("service.notification_status"),
     }
}
func (r Resource)  GetNotification_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_statusService -> GetNotification_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetNotification_status.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetNotification_status.offset", offset)
	tracker.AddParam("service.GetNotification_status.limit", limit)



	itemsPage, err = r.notification_statusRepo.GetNotification_status(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Notification_status); ok {
		tracker.AddResult("service.GetNotification_status.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetNotification_statusById(ctx context.Context, id int64) (notification_status *model.Notification_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_statusService -> GetNotification_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetNotification_statusById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetNotification_statusById.id", id)
	notification_status, err = r.notification_statusRepo.GetNotification_statusById(ctx, id)
	if err != nil {
		return notification_status, errors.New(app.MsgRepositoryError)
	}

	return notification_status, nil
}
func (r Resource)  GetNotification_statusByStatusCode(ctx context.Context, statuscode string) (notification_status *model.Notification_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_statusService -> GetNotification_statusByStatusCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetNotification_statusByStatusCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetNotification_statusByStatusCode.statuscode", statuscode)
	notification_status, err = r.notification_statusRepo.GetNotification_statusByStatusCode(ctx, statuscode)
	if err != nil {
		return notification_status, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetNotification_statusByStatusCode.found", notification_status != nil)
	return notification_status, nil
}
func (r Resource)  DeleteNotification_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_statusService -> DeleteNotification_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteNotification_statusById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteNotification_statusById.id",id)

	result, err = r.notification_statusRepo.DeleteNotification_statusById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteNotification_statusById.deleted", result)
	return result, nil
}
func (r Resource)  InsertNotification_status(ctx context.Context,notification_status *model.Notification_status) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_statusService -> InsertNotification_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertNotification_status")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertNotification_status.statuscode", notification_status.StatusCode)
	insertedId, err = r.notification_statusRepo.InsertNotification_status(ctx, notification_status)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertNotification_status.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateNotification_status(ctx context.Context,notification_status *model.Notification_status, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_statusService -> UpdateNotification_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateNotification_status")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateNotification_status.id", id)
	tracker.AddParam("service.UpdateNotification_status.statuscode", notification_status.StatusCode)

	err = r.notification_statusRepo.UpdateNotification_status(ctx, notification_status, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateNotification_status.updated", true)

	return nil
}

