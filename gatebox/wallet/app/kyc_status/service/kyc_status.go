package kyc_statusSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	kyc_statusRepo "palm-pay/app/kyc_status/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Kyc_statusServiceIF interface {
     GetKyc_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetKyc_statusById(ctx context.Context, id int64) (*model.Kyc_status, error)
     GetKyc_statusByStatusCode(ctx context.Context, statuscode string) (*model.Kyc_status, error)
     InsertKyc_status(ctx context.Context, kyc_status *model.Kyc_status) (int64, error)
     UpdateKyc_status(ctx context.Context, kyc_status *model.Kyc_status, id int64) error
     DeleteKyc_statusById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     kyc_statusRepo kyc_statusRepo.Kyc_statusRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewKyc_statusService(kyc_statusRepo kyc_statusRepo.Kyc_statusRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         kyc_statusRepo: kyc_statusRepo,
		  observability:  observabilidade.NewServiceObservability("service.kyc_status"),
     }
}
func (r Resource)  GetKyc_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Kyc_statusService -> GetKyc_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetKyc_status.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetKyc_status.offset", offset)
	tracker.AddParam("service.GetKyc_status.limit", limit)



	itemsPage, err = r.kyc_statusRepo.GetKyc_status(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Kyc_status); ok {
		tracker.AddResult("service.GetKyc_status.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetKyc_statusById(ctx context.Context, id int64) (kyc_status *model.Kyc_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Kyc_statusService -> GetKyc_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetKyc_statusById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetKyc_statusById.id", id)
	kyc_status, err = r.kyc_statusRepo.GetKyc_statusById(ctx, id)
	if err != nil {
		return kyc_status, errors.New(app.MsgRepositoryError)
	}

	return kyc_status, nil
}
func (r Resource)  GetKyc_statusByStatusCode(ctx context.Context, statuscode string) (kyc_status *model.Kyc_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Kyc_statusService -> GetKyc_statusByStatusCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetKyc_statusByStatusCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetKyc_statusByStatusCode.statuscode", statuscode)
	kyc_status, err = r.kyc_statusRepo.GetKyc_statusByStatusCode(ctx, statuscode)
	if err != nil {
		return kyc_status, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetKyc_statusByStatusCode.found", kyc_status != nil)
	return kyc_status, nil
}
func (r Resource)  DeleteKyc_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Kyc_statusService -> DeleteKyc_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteKyc_statusById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteKyc_statusById.id",id)

	result, err = r.kyc_statusRepo.DeleteKyc_statusById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteKyc_statusById.deleted", result)
	return result, nil
}
func (r Resource)  InsertKyc_status(ctx context.Context,kyc_status *model.Kyc_status) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Kyc_statusService -> InsertKyc_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertKyc_status")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertKyc_status.statuscode", kyc_status.StatusCode)
	insertedId, err = r.kyc_statusRepo.InsertKyc_status(ctx, kyc_status)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertKyc_status.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateKyc_status(ctx context.Context,kyc_status *model.Kyc_status, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Kyc_statusService -> UpdateKyc_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateKyc_status")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateKyc_status.id", id)
	tracker.AddParam("service.UpdateKyc_status.statuscode", kyc_status.StatusCode)

	err = r.kyc_statusRepo.UpdateKyc_status(ctx, kyc_status, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateKyc_status.updated", true)

	return nil
}

