package acquirersSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	acquirersRepo "palm-pay/app/acquirers/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type AcquirersServiceIF interface {
     GetAcquirers(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAcquirersById(ctx context.Context, id int64) (*model.Acquirers, error)
     GetAcquirersByAcquirerCode(ctx context.Context, acquirercode string) (*model.Acquirers, error)
     InsertAcquirers(ctx context.Context, acquirers *model.Acquirers) (int64, error)
     UpdateAcquirers(ctx context.Context, acquirers *model.Acquirers, id int64) error
     DeleteAcquirersById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     acquirersRepo acquirersRepo.AcquirersRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewAcquirersService(acquirersRepo acquirersRepo.AcquirersRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         acquirersRepo: acquirersRepo,
		  observability:  observabilidade.NewServiceObservability("service.acquirers"),
     }
}
func (r Resource)  GetAcquirers(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("AcquirersService -> GetAcquirers", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAcquirers.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetAcquirers.offset", offset)
	tracker.AddParam("service.GetAcquirers.limit", limit)



	itemsPage, err = r.acquirersRepo.GetAcquirers(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Acquirers); ok {
		tracker.AddResult("service.GetAcquirers.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetAcquirersById(ctx context.Context, id int64) (acquirers *model.Acquirers, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("AcquirersService -> GetAcquirersById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAcquirersById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetAcquirersById.id", id)
	acquirers, err = r.acquirersRepo.GetAcquirersById(ctx, id)
	if err != nil {
		return acquirers, errors.New(app.MsgRepositoryError)
	}

	return acquirers, nil
}
func (r Resource)  GetAcquirersByAcquirerCode(ctx context.Context, acquirercode string) (acquirers *model.Acquirers, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("AcquirersService -> GetAcquirersByAcquirerCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetAcquirersByAcquirerCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetAcquirersByAcquirerCode.acquirercode", acquirercode)
	acquirers, err = r.acquirersRepo.GetAcquirersByAcquirerCode(ctx, acquirercode)
	if err != nil {
		return acquirers, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetAcquirersByAcquirerCode.found", acquirers != nil)
	return acquirers, nil
}
func (r Resource)  DeleteAcquirersById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("AcquirersService -> DeleteAcquirersById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteAcquirersById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteAcquirersById.id",id)

	result, err = r.acquirersRepo.DeleteAcquirersById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteAcquirersById.deleted", result)
	return result, nil
}
func (r Resource)  InsertAcquirers(ctx context.Context,acquirers *model.Acquirers) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("AcquirersService -> InsertAcquirers", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertAcquirers")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertAcquirers.acquirercode", acquirers.AcquirerCode)
	insertedId, err = r.acquirersRepo.InsertAcquirers(ctx, acquirers)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertAcquirers.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateAcquirers(ctx context.Context,acquirers *model.Acquirers, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("AcquirersService -> UpdateAcquirers", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateAcquirers")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateAcquirers.id", id)
	tracker.AddParam("service.UpdateAcquirers.acquirercode", acquirers.AcquirerCode)

	err = r.acquirersRepo.UpdateAcquirers(ctx, acquirers, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateAcquirers.updated", true)

	return nil
}

