package notification_templatesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	notification_templatesRepo "palm-pay/app/notification_templates/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Notification_templatesServiceIF interface {
     GetNotification_templates(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetNotification_templatesById(ctx context.Context, id int64) (*model.Notification_templates, error)
     GetNotification_templatesByTemplateCode(ctx context.Context, templatecode string) (*model.Notification_templates, error)
     InsertNotification_templates(ctx context.Context, notification_templates *model.Notification_templates) (int64, error)
     UpdateNotification_templates(ctx context.Context, notification_templates *model.Notification_templates, id int64) error
     DeleteNotification_templatesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     notification_templatesRepo notification_templatesRepo.Notification_templatesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewNotification_templatesService(notification_templatesRepo notification_templatesRepo.Notification_templatesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         notification_templatesRepo: notification_templatesRepo,
		  observability:  observabilidade.NewServiceObservability("service.notification_templates"),
     }
}
func (r Resource)  GetNotification_templates(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_templatesService -> GetNotification_templates", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetNotification_templates.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetNotification_templates.offset", offset)
	tracker.AddParam("service.GetNotification_templates.limit", limit)



	itemsPage, err = r.notification_templatesRepo.GetNotification_templates(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Notification_templates); ok {
		tracker.AddResult("service.GetNotification_templates.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetNotification_templatesById(ctx context.Context, id int64) (notification_templates *model.Notification_templates, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_templatesService -> GetNotification_templatesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetNotification_templatesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetNotification_templatesById.id", id)
	notification_templates, err = r.notification_templatesRepo.GetNotification_templatesById(ctx, id)
	if err != nil {
		return notification_templates, errors.New(app.MsgRepositoryError)
	}

	return notification_templates, nil
}
func (r Resource)  GetNotification_templatesByTemplateCode(ctx context.Context, templatecode string) (notification_templates *model.Notification_templates, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_templatesService -> GetNotification_templatesByTemplateCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetNotification_templatesByTemplateCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetNotification_templatesByTemplateCode.templatecode", templatecode)
	notification_templates, err = r.notification_templatesRepo.GetNotification_templatesByTemplateCode(ctx, templatecode)
	if err != nil {
		return notification_templates, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetNotification_templatesByTemplateCode.found", notification_templates != nil)
	return notification_templates, nil
}
func (r Resource)  DeleteNotification_templatesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_templatesService -> DeleteNotification_templatesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteNotification_templatesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteNotification_templatesById.id",id)

	result, err = r.notification_templatesRepo.DeleteNotification_templatesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteNotification_templatesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertNotification_templates(ctx context.Context,notification_templates *model.Notification_templates) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_templatesService -> InsertNotification_templates", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertNotification_templates")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertNotification_templates.templatecode", notification_templates.TemplateCode)
	insertedId, err = r.notification_templatesRepo.InsertNotification_templates(ctx, notification_templates)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertNotification_templates.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateNotification_templates(ctx context.Context,notification_templates *model.Notification_templates, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Notification_templatesService -> UpdateNotification_templates", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateNotification_templates")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateNotification_templates.id", id)
	tracker.AddParam("service.UpdateNotification_templates.templatecode", notification_templates.TemplateCode)

	err = r.notification_templatesRepo.UpdateNotification_templates(ctx, notification_templates, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateNotification_templates.updated", true)

	return nil
}

