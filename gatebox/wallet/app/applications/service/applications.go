package applicationsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	applicationsRepo "palm-pay/app/applications/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type ApplicationsServiceIF interface {
     GetApplications(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetApplicationsById(ctx context.Context, id int64) (*model.Applications, error)
     GetApplicationsByAppCode(ctx context.Context, appcode string) (*model.Applications, error)
     InsertApplications(ctx context.Context, applications *model.Applications) (int64, error)
     UpdateApplications(ctx context.Context, applications *model.Applications, id int64) error
     DeleteApplicationsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     applicationsRepo applicationsRepo.ApplicationsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewApplicationsService(applicationsRepo applicationsRepo.ApplicationsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         applicationsRepo: applicationsRepo,
		  observability:  observabilidade.NewServiceObservability("service.applications"),
     }
}
func (r Resource)  GetApplications(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("ApplicationsService -> GetApplications", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetApplications.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetApplications.offset", offset)
	tracker.AddParam("service.GetApplications.limit", limit)



	itemsPage, err = r.applicationsRepo.GetApplications(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Applications); ok {
		tracker.AddResult("service.GetApplications.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetApplicationsById(ctx context.Context, id int64) (applications *model.Applications, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("ApplicationsService -> GetApplicationsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetApplicationsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetApplicationsById.id", id)
	applications, err = r.applicationsRepo.GetApplicationsById(ctx, id)
	if err != nil {
		return applications, errors.New(app.MsgRepositoryError)
	}

	return applications, nil
}
func (r Resource)  GetApplicationsByAppCode(ctx context.Context, appcode string) (applications *model.Applications, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("ApplicationsService -> GetApplicationsByAppCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetApplicationsByAppCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetApplicationsByAppCode.appcode", appcode)
	applications, err = r.applicationsRepo.GetApplicationsByAppCode(ctx, appcode)
	if err != nil {
		return applications, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetApplicationsByAppCode.found", applications != nil)
	return applications, nil
}
func (r Resource)  DeleteApplicationsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("ApplicationsService -> DeleteApplicationsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteApplicationsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteApplicationsById.id",id)

	result, err = r.applicationsRepo.DeleteApplicationsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteApplicationsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertApplications(ctx context.Context,applications *model.Applications) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("ApplicationsService -> InsertApplications", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertApplications")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertApplications.appcode", applications.AppCode)
	insertedId, err = r.applicationsRepo.InsertApplications(ctx, applications)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertApplications.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateApplications(ctx context.Context,applications *model.Applications, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("ApplicationsService -> UpdateApplications", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateApplications")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateApplications.id", id)
	tracker.AddParam("service.UpdateApplications.appcode", applications.AppCode)

	err = r.applicationsRepo.UpdateApplications(ctx, applications, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateApplications.updated", true)

	return nil
}

