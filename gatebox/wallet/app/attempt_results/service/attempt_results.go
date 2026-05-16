package attempt_resultsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	attempt_resultsRepo "palm-pay/app/attempt_results/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Attempt_resultsServiceIF interface {
     GetAttempt_results(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAttempt_resultsById(ctx context.Context, id int64) (*model.Attempt_results, error)
     GetAttempt_resultsByResultCode(ctx context.Context, resultcode string) (*model.Attempt_results, error)
     InsertAttempt_results(ctx context.Context, attempt_results *model.Attempt_results) (int64, error)
     UpdateAttempt_results(ctx context.Context, attempt_results *model.Attempt_results, id int64) error
     DeleteAttempt_resultsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     attempt_resultsRepo attempt_resultsRepo.Attempt_resultsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewAttempt_resultsService(attempt_resultsRepo attempt_resultsRepo.Attempt_resultsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         attempt_resultsRepo: attempt_resultsRepo,
		  observability:  observabilidade.NewServiceObservability("service.attempt_results"),
     }
}
func (r Resource)  GetAttempt_results(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Attempt_resultsService -> GetAttempt_results", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAttempt_results.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetAttempt_results.offset", offset)
	tracker.AddParam("service.GetAttempt_results.limit", limit)



	itemsPage, err = r.attempt_resultsRepo.GetAttempt_results(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Attempt_results); ok {
		tracker.AddResult("service.GetAttempt_results.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetAttempt_resultsById(ctx context.Context, id int64) (attempt_results *model.Attempt_results, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Attempt_resultsService -> GetAttempt_resultsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAttempt_resultsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetAttempt_resultsById.id", id)
	attempt_results, err = r.attempt_resultsRepo.GetAttempt_resultsById(ctx, id)
	if err != nil {
		return attempt_results, errors.New(app.MsgRepositoryError)
	}

	return attempt_results, nil
}
func (r Resource)  GetAttempt_resultsByResultCode(ctx context.Context, resultcode string) (attempt_results *model.Attempt_results, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Attempt_resultsService -> GetAttempt_resultsByResultCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetAttempt_resultsByResultCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetAttempt_resultsByResultCode.resultcode", resultcode)
	attempt_results, err = r.attempt_resultsRepo.GetAttempt_resultsByResultCode(ctx, resultcode)
	if err != nil {
		return attempt_results, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetAttempt_resultsByResultCode.found", attempt_results != nil)
	return attempt_results, nil
}
func (r Resource)  DeleteAttempt_resultsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Attempt_resultsService -> DeleteAttempt_resultsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteAttempt_resultsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteAttempt_resultsById.id",id)

	result, err = r.attempt_resultsRepo.DeleteAttempt_resultsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteAttempt_resultsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertAttempt_results(ctx context.Context,attempt_results *model.Attempt_results) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Attempt_resultsService -> InsertAttempt_results", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertAttempt_results")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertAttempt_results.resultcode", attempt_results.ResultCode)
	insertedId, err = r.attempt_resultsRepo.InsertAttempt_results(ctx, attempt_results)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertAttempt_results.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateAttempt_results(ctx context.Context,attempt_results *model.Attempt_results, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Attempt_resultsService -> UpdateAttempt_results", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateAttempt_results")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateAttempt_results.id", id)
	tracker.AddParam("service.UpdateAttempt_results.resultcode", attempt_results.ResultCode)

	err = r.attempt_resultsRepo.UpdateAttempt_results(ctx, attempt_results, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateAttempt_results.updated", true)

	return nil
}

