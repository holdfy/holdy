package notification_channelsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	notification_channelsRepo "palm-pay/app/notification_channels/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Notification_channelsServiceIF interface {
     GetNotification_channels(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetNotification_channelsById(ctx context.Context, id int64) (*model.Notification_channels, error)
     GetNotification_channelsByChannelCode(ctx context.Context, channelcode string) (*model.Notification_channels, error)
     InsertNotification_channels(ctx context.Context, notification_channels *model.Notification_channels) (int64, error)
     UpdateNotification_channels(ctx context.Context, notification_channels *model.Notification_channels, id int64) error
     DeleteNotification_channelsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     notification_channelsRepo notification_channelsRepo.Notification_channelsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewNotification_channelsService(notification_channelsRepo notification_channelsRepo.Notification_channelsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         notification_channelsRepo: notification_channelsRepo,
		  observability:  observabilidade.NewServiceObservability("service.notification_channels"),
     }
}
func (r Resource)  GetNotification_channels(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_channelsService -> GetNotification_channels", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetNotification_channels.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetNotification_channels.offset", offset)
	tracker.AddParam("service.GetNotification_channels.limit", limit)



	itemsPage, err = r.notification_channelsRepo.GetNotification_channels(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Notification_channels); ok {
		tracker.AddResult("service.GetNotification_channels.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetNotification_channelsById(ctx context.Context, id int64) (notification_channels *model.Notification_channels, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_channelsService -> GetNotification_channelsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetNotification_channelsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetNotification_channelsById.id", id)
	notification_channels, err = r.notification_channelsRepo.GetNotification_channelsById(ctx, id)
	if err != nil {
		return notification_channels, errors.New(app.MsgRepositoryError)
	}

	return notification_channels, nil
}
func (r Resource)  GetNotification_channelsByChannelCode(ctx context.Context, channelcode string) (notification_channels *model.Notification_channels, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_channelsService -> GetNotification_channelsByChannelCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetNotification_channelsByChannelCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetNotification_channelsByChannelCode.channelcode", channelcode)
	notification_channels, err = r.notification_channelsRepo.GetNotification_channelsByChannelCode(ctx, channelcode)
	if err != nil {
		return notification_channels, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetNotification_channelsByChannelCode.found", notification_channels != nil)
	return notification_channels, nil
}
func (r Resource)  DeleteNotification_channelsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_channelsService -> DeleteNotification_channelsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteNotification_channelsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteNotification_channelsById.id",id)

	result, err = r.notification_channelsRepo.DeleteNotification_channelsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteNotification_channelsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertNotification_channels(ctx context.Context,notification_channels *model.Notification_channels) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_channelsService -> InsertNotification_channels", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertNotification_channels")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertNotification_channels.channelcode", notification_channels.ChannelCode)
	insertedId, err = r.notification_channelsRepo.InsertNotification_channels(ctx, notification_channels)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertNotification_channels.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateNotification_channels(ctx context.Context,notification_channels *model.Notification_channels, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_channelsService -> UpdateNotification_channels", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateNotification_channels")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateNotification_channels.id", id)
	tracker.AddParam("service.UpdateNotification_channels.channelcode", notification_channels.ChannelCode)

	err = r.notification_channelsRepo.UpdateNotification_channels(ctx, notification_channels, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateNotification_channels.updated", true)

	return nil
}

