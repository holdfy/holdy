package biometric_attemptsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	biometric_attemptsRepo "palm-pay/app/biometric_attempts/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Biometric_attemptsServiceIF interface {
     GetBiometric_attempts(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetBiometric_attemptsById(ctx context.Context, id int64) (*model.Biometric_attempts, error)
     GetBiometric_attemptsByAttemptCode(ctx context.Context, attemptcode string) (*model.Biometric_attempts, error)
     InsertBiometric_attempts(ctx context.Context, biometric_attempts *model.Biometric_attempts) (int64, error)
     UpdateBiometric_attempts(ctx context.Context, biometric_attempts *model.Biometric_attempts, id int64) error
     DeleteBiometric_attemptsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     biometric_attemptsRepo biometric_attemptsRepo.Biometric_attemptsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewBiometric_attemptsService(biometric_attemptsRepo biometric_attemptsRepo.Biometric_attemptsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         biometric_attemptsRepo: biometric_attemptsRepo,
		  observability:  observabilidade.NewServiceObservability("service.biometric_attempts"),
     }
}
func (r Resource)  GetBiometric_attempts(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Biometric_attemptsService -> GetBiometric_attempts", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetBiometric_attempts.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetBiometric_attempts.offset", offset)
	tracker.AddParam("service.GetBiometric_attempts.limit", limit)



	itemsPage, err = r.biometric_attemptsRepo.GetBiometric_attempts(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Biometric_attempts); ok {
		tracker.AddResult("service.GetBiometric_attempts.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetBiometric_attemptsById(ctx context.Context, id int64) (biometric_attempts *model.Biometric_attempts, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Biometric_attemptsService -> GetBiometric_attemptsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetBiometric_attemptsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetBiometric_attemptsById.id", id)
	biometric_attempts, err = r.biometric_attemptsRepo.GetBiometric_attemptsById(ctx, id)
	if err != nil {
		return biometric_attempts, errors.New(app.MsgRepositoryError)
	}

	return biometric_attempts, nil
}
func (r Resource)  GetBiometric_attemptsByAttemptCode(ctx context.Context, attemptcode string) (biometric_attempts *model.Biometric_attempts, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Biometric_attemptsService -> GetBiometric_attemptsByAttemptCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetBiometric_attemptsByAttemptCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetBiometric_attemptsByAttemptCode.attemptcode", attemptcode)
	biometric_attempts, err = r.biometric_attemptsRepo.GetBiometric_attemptsByAttemptCode(ctx, attemptcode)
	if err != nil {
		return biometric_attempts, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetBiometric_attemptsByAttemptCode.found", biometric_attempts != nil)
	return biometric_attempts, nil
}
func (r Resource)  DeleteBiometric_attemptsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Biometric_attemptsService -> DeleteBiometric_attemptsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteBiometric_attemptsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteBiometric_attemptsById.id",id)

	result, err = r.biometric_attemptsRepo.DeleteBiometric_attemptsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteBiometric_attemptsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertBiometric_attempts(ctx context.Context,biometric_attempts *model.Biometric_attempts) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Biometric_attemptsService -> InsertBiometric_attempts", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertBiometric_attempts")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertBiometric_attempts.attemptcode", biometric_attempts.AttemptCode)
	insertedId, err = r.biometric_attemptsRepo.InsertBiometric_attempts(ctx, biometric_attempts)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertBiometric_attempts.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateBiometric_attempts(ctx context.Context,biometric_attempts *model.Biometric_attempts, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Biometric_attemptsService -> UpdateBiometric_attempts", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateBiometric_attempts")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateBiometric_attempts.id", id)
	tracker.AddParam("service.UpdateBiometric_attempts.attemptcode", biometric_attempts.AttemptCode)

	err = r.biometric_attemptsRepo.UpdateBiometric_attempts(ctx, biometric_attempts, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateBiometric_attempts.updated", true)

	return nil
}

