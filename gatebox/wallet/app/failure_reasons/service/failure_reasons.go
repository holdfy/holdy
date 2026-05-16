package failure_reasonsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	failure_reasonsRepo "palm-pay/app/failure_reasons/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Failure_reasonsServiceIF interface {
     GetFailure_reasons(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetFailure_reasonsById(ctx context.Context, id int64) (*model.Failure_reasons, error)
     GetFailure_reasonsByReasonCode(ctx context.Context, reasoncode string) (*model.Failure_reasons, error)
     InsertFailure_reasons(ctx context.Context, failure_reasons *model.Failure_reasons) (int64, error)
     UpdateFailure_reasons(ctx context.Context, failure_reasons *model.Failure_reasons, id int64) error
     DeleteFailure_reasonsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     failure_reasonsRepo failure_reasonsRepo.Failure_reasonsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewFailure_reasonsService(failure_reasonsRepo failure_reasonsRepo.Failure_reasonsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         failure_reasonsRepo: failure_reasonsRepo,
		  observability:  observabilidade.NewServiceObservability("service.failure_reasons"),
     }
}
func (r Resource)  GetFailure_reasons(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Failure_reasonsService -> GetFailure_reasons", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetFailure_reasons.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetFailure_reasons.offset", offset)
	tracker.AddParam("service.GetFailure_reasons.limit", limit)



	itemsPage, err = r.failure_reasonsRepo.GetFailure_reasons(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Failure_reasons); ok {
		tracker.AddResult("service.GetFailure_reasons.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetFailure_reasonsById(ctx context.Context, id int64) (failure_reasons *model.Failure_reasons, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Failure_reasonsService -> GetFailure_reasonsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetFailure_reasonsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetFailure_reasonsById.id", id)
	failure_reasons, err = r.failure_reasonsRepo.GetFailure_reasonsById(ctx, id)
	if err != nil {
		return failure_reasons, errors.New(app.MsgRepositoryError)
	}

	return failure_reasons, nil
}
func (r Resource)  GetFailure_reasonsByReasonCode(ctx context.Context, reasoncode string) (failure_reasons *model.Failure_reasons, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Failure_reasonsService -> GetFailure_reasonsByReasonCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetFailure_reasonsByReasonCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetFailure_reasonsByReasonCode.reasoncode", reasoncode)
	failure_reasons, err = r.failure_reasonsRepo.GetFailure_reasonsByReasonCode(ctx, reasoncode)
	if err != nil {
		return failure_reasons, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetFailure_reasonsByReasonCode.found", failure_reasons != nil)
	return failure_reasons, nil
}
func (r Resource)  DeleteFailure_reasonsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Failure_reasonsService -> DeleteFailure_reasonsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteFailure_reasonsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteFailure_reasonsById.id",id)

	result, err = r.failure_reasonsRepo.DeleteFailure_reasonsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteFailure_reasonsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertFailure_reasons(ctx context.Context,failure_reasons *model.Failure_reasons) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Failure_reasonsService -> InsertFailure_reasons", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertFailure_reasons")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertFailure_reasons.reasoncode", failure_reasons.ReasonCode)
	insertedId, err = r.failure_reasonsRepo.InsertFailure_reasons(ctx, failure_reasons)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertFailure_reasons.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateFailure_reasons(ctx context.Context,failure_reasons *model.Failure_reasons, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Failure_reasonsService -> UpdateFailure_reasons", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateFailure_reasons")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateFailure_reasons.id", id)
	tracker.AddParam("service.UpdateFailure_reasons.reasoncode", failure_reasons.ReasonCode)

	err = r.failure_reasonsRepo.UpdateFailure_reasons(ctx, failure_reasons, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateFailure_reasons.updated", true)

	return nil
}

