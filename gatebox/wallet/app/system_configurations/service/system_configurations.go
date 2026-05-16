package system_configurationsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	system_configurationsRepo "palm-pay/app/system_configurations/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type System_configurationsServiceIF interface {
     GetSystem_configurations(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSystem_configurationsById(ctx context.Context, id int64) (*model.System_configurations, error)
     GetSystem_configurationsByConfigCode(ctx context.Context, configcode string) (*model.System_configurations, error)
     InsertSystem_configurations(ctx context.Context, system_configurations *model.System_configurations) (int64, error)
     UpdateSystem_configurations(ctx context.Context, system_configurations *model.System_configurations, id int64) error
     DeleteSystem_configurationsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     system_configurationsRepo system_configurationsRepo.System_configurationsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewSystem_configurationsService(system_configurationsRepo system_configurationsRepo.System_configurationsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         system_configurationsRepo: system_configurationsRepo,
		  observability:  observabilidade.NewServiceObservability("service.system_configurations"),
     }
}
func (r Resource)  GetSystem_configurations(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("System_configurationsService -> GetSystem_configurations", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSystem_configurations.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetSystem_configurations.offset", offset)
	tracker.AddParam("service.GetSystem_configurations.limit", limit)



	itemsPage, err = r.system_configurationsRepo.GetSystem_configurations(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.System_configurations); ok {
		tracker.AddResult("service.GetSystem_configurations.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetSystem_configurationsById(ctx context.Context, id int64) (system_configurations *model.System_configurations, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("System_configurationsService -> GetSystem_configurationsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSystem_configurationsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetSystem_configurationsById.id", id)
	system_configurations, err = r.system_configurationsRepo.GetSystem_configurationsById(ctx, id)
	if err != nil {
		return system_configurations, errors.New(app.MsgRepositoryError)
	}

	return system_configurations, nil
}
func (r Resource)  GetSystem_configurationsByConfigCode(ctx context.Context, configcode string) (system_configurations *model.System_configurations, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("System_configurationsService -> GetSystem_configurationsByConfigCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetSystem_configurationsByConfigCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetSystem_configurationsByConfigCode.configcode", configcode)
	system_configurations, err = r.system_configurationsRepo.GetSystem_configurationsByConfigCode(ctx, configcode)
	if err != nil {
		return system_configurations, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetSystem_configurationsByConfigCode.found", system_configurations != nil)
	return system_configurations, nil
}
func (r Resource)  DeleteSystem_configurationsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("System_configurationsService -> DeleteSystem_configurationsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteSystem_configurationsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteSystem_configurationsById.id",id)

	result, err = r.system_configurationsRepo.DeleteSystem_configurationsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteSystem_configurationsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertSystem_configurations(ctx context.Context,system_configurations *model.System_configurations) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("System_configurationsService -> InsertSystem_configurations", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertSystem_configurations")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertSystem_configurations.configcode", system_configurations.ConfigCode)
	insertedId, err = r.system_configurationsRepo.InsertSystem_configurations(ctx, system_configurations)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertSystem_configurations.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateSystem_configurations(ctx context.Context,system_configurations *model.System_configurations, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("System_configurationsService -> UpdateSystem_configurations", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateSystem_configurations")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateSystem_configurations.id", id)
	tracker.AddParam("service.UpdateSystem_configurations.configcode", system_configurations.ConfigCode)

	err = r.system_configurationsRepo.UpdateSystem_configurations(ctx, system_configurations, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateSystem_configurations.updated", true)

	return nil
}

