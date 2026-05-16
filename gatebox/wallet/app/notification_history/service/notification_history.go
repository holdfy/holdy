package notification_historySV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	notification_historyRepo "palm-pay/app/notification_history/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Notification_historyServiceIF interface {
     GetNotification_history(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetNotification_historyById(ctx context.Context, id int64) (*model.Notification_history, error)
     GetNotification_historyByNotificationCode(ctx context.Context, notificationcode string) (*model.Notification_history, error)
     InsertNotification_history(ctx context.Context, notification_history *model.Notification_history) (int64, error)
     UpdateNotification_history(ctx context.Context, notification_history *model.Notification_history, id int64) error
     DeleteNotification_historyById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     notification_historyRepo notification_historyRepo.Notification_historyRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewNotification_historyService(notification_historyRepo notification_historyRepo.Notification_historyRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         notification_historyRepo: notification_historyRepo,
		  observability:  observabilidade.NewServiceObservability("service.notification_history"),
     }
}
func (r Resource)  GetNotification_history(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_historyService -> GetNotification_history", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetNotification_history.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetNotification_history.offset", offset)
	tracker.AddParam("service.GetNotification_history.limit", limit)



	itemsPage, err = r.notification_historyRepo.GetNotification_history(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Notification_history); ok {
		tracker.AddResult("service.GetNotification_history.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetNotification_historyById(ctx context.Context, id int64) (notification_history *model.Notification_history, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_historyService -> GetNotification_historyById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetNotification_historyById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetNotification_historyById.id", id)
	notification_history, err = r.notification_historyRepo.GetNotification_historyById(ctx, id)
	if err != nil {
		return notification_history, errors.New(app.MsgRepositoryError)
	}

	return notification_history, nil
}
func (r Resource)  GetNotification_historyByNotificationCode(ctx context.Context, notificationcode string) (notification_history *model.Notification_history, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_historyService -> GetNotification_historyByNotificationCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetNotification_historyByNotificationCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetNotification_historyByNotificationCode.notificationcode", notificationcode)
	notification_history, err = r.notification_historyRepo.GetNotification_historyByNotificationCode(ctx, notificationcode)
	if err != nil {
		return notification_history, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetNotification_historyByNotificationCode.found", notification_history != nil)
	return notification_history, nil
}
func (r Resource)  DeleteNotification_historyById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_historyService -> DeleteNotification_historyById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteNotification_historyById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteNotification_historyById.id",id)

	result, err = r.notification_historyRepo.DeleteNotification_historyById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteNotification_historyById.deleted", result)
	return result, nil
}
func (r Resource)  InsertNotification_history(ctx context.Context,notification_history *model.Notification_history) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_historyService -> InsertNotification_history", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertNotification_history")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertNotification_history.notificationcode", notification_history.NotificationCode)
	insertedId, err = r.notification_historyRepo.InsertNotification_history(ctx, notification_history)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertNotification_history.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateNotification_history(ctx context.Context,notification_history *model.Notification_history, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_historyService -> UpdateNotification_history", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateNotification_history")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateNotification_history.id", id)
	tracker.AddParam("service.UpdateNotification_history.notificationcode", notification_history.NotificationCode)

	err = r.notification_historyRepo.UpdateNotification_history(ctx, notification_history, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateNotification_history.updated", true)

	return nil
}

