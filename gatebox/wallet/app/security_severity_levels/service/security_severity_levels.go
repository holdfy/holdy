package security_severity_levelsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	security_severity_levelsRepo "palm-pay/app/security_severity_levels/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Security_severity_levelsServiceIF interface {
     GetSecurity_severity_levels(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSecurity_severity_levelsById(ctx context.Context, id int64) (*model.Security_severity_levels, error)
     GetSecurity_severity_levelsBySeverityCode(ctx context.Context, severitycode string) (*model.Security_severity_levels, error)
     InsertSecurity_severity_levels(ctx context.Context, security_severity_levels *model.Security_severity_levels) (int64, error)
     UpdateSecurity_severity_levels(ctx context.Context, security_severity_levels *model.Security_severity_levels, id int64) error
     DeleteSecurity_severity_levelsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     security_severity_levelsRepo security_severity_levelsRepo.Security_severity_levelsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewSecurity_severity_levelsService(security_severity_levelsRepo security_severity_levelsRepo.Security_severity_levelsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         security_severity_levelsRepo: security_severity_levelsRepo,
		  observability:  observabilidade.NewServiceObservability("service.security_severity_levels"),
     }
}
func (r Resource)  GetSecurity_severity_levels(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_severity_levelsService -> GetSecurity_severity_levels", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSecurity_severity_levels.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetSecurity_severity_levels.offset", offset)
	tracker.AddParam("service.GetSecurity_severity_levels.limit", limit)



	itemsPage, err = r.security_severity_levelsRepo.GetSecurity_severity_levels(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Security_severity_levels); ok {
		tracker.AddResult("service.GetSecurity_severity_levels.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetSecurity_severity_levelsById(ctx context.Context, id int64) (security_severity_levels *model.Security_severity_levels, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_severity_levelsService -> GetSecurity_severity_levelsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSecurity_severity_levelsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetSecurity_severity_levelsById.id", id)
	security_severity_levels, err = r.security_severity_levelsRepo.GetSecurity_severity_levelsById(ctx, id)
	if err != nil {
		return security_severity_levels, errors.New(app.MsgRepositoryError)
	}

	return security_severity_levels, nil
}
func (r Resource)  GetSecurity_severity_levelsBySeverityCode(ctx context.Context, severitycode string) (security_severity_levels *model.Security_severity_levels, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_severity_levelsService -> GetSecurity_severity_levelsBySeverityCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetSecurity_severity_levelsBySeverityCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetSecurity_severity_levelsBySeverityCode.severitycode", severitycode)
	security_severity_levels, err = r.security_severity_levelsRepo.GetSecurity_severity_levelsBySeverityCode(ctx, severitycode)
	if err != nil {
		return security_severity_levels, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetSecurity_severity_levelsBySeverityCode.found", security_severity_levels != nil)
	return security_severity_levels, nil
}
func (r Resource)  DeleteSecurity_severity_levelsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_severity_levelsService -> DeleteSecurity_severity_levelsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteSecurity_severity_levelsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteSecurity_severity_levelsById.id",id)

	result, err = r.security_severity_levelsRepo.DeleteSecurity_severity_levelsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteSecurity_severity_levelsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertSecurity_severity_levels(ctx context.Context,security_severity_levels *model.Security_severity_levels) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_severity_levelsService -> InsertSecurity_severity_levels", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertSecurity_severity_levels")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertSecurity_severity_levels.severitycode", security_severity_levels.SeverityCode)
	insertedId, err = r.security_severity_levelsRepo.InsertSecurity_severity_levels(ctx, security_severity_levels)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertSecurity_severity_levels.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateSecurity_severity_levels(ctx context.Context,security_severity_levels *model.Security_severity_levels, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_severity_levelsService -> UpdateSecurity_severity_levels", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateSecurity_severity_levels")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateSecurity_severity_levels.id", id)
	tracker.AddParam("service.UpdateSecurity_severity_levels.severitycode", security_severity_levels.SeverityCode)

	err = r.security_severity_levelsRepo.UpdateSecurity_severity_levels(ctx, security_severity_levels, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateSecurity_severity_levels.updated", true)

	return nil
}

