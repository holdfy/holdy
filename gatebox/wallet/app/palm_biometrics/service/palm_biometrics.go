package palm_biometricsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	palm_biometricsRepo "palm-pay/app/palm_biometrics/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Palm_biometricsServiceIF interface {
     GetPalm_biometrics(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetPalm_biometricsById(ctx context.Context, id int64) (*model.Palm_biometrics, error)
     GetPalm_biometricsByBiometricCode(ctx context.Context, biometriccode string) (*model.Palm_biometrics, error)
     InsertPalm_biometrics(ctx context.Context, palm_biometrics *model.Palm_biometrics) (int64, error)
     UpdatePalm_biometrics(ctx context.Context, palm_biometrics *model.Palm_biometrics, id int64) error
     DeletePalm_biometricsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     palm_biometricsRepo palm_biometricsRepo.Palm_biometricsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewPalm_biometricsService(palm_biometricsRepo palm_biometricsRepo.Palm_biometricsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         palm_biometricsRepo: palm_biometricsRepo,
		  observability:  observabilidade.NewServiceObservability("service.palm_biometrics"),
     }
}
func (r Resource)  GetPalm_biometrics(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Palm_biometricsService -> GetPalm_biometrics", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetPalm_biometrics.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetPalm_biometrics.offset", offset)
	tracker.AddParam("service.GetPalm_biometrics.limit", limit)



	itemsPage, err = r.palm_biometricsRepo.GetPalm_biometrics(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Palm_biometrics); ok {
		tracker.AddResult("service.GetPalm_biometrics.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetPalm_biometricsById(ctx context.Context, id int64) (palm_biometrics *model.Palm_biometrics, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Palm_biometricsService -> GetPalm_biometricsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetPalm_biometricsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetPalm_biometricsById.id", id)
	palm_biometrics, err = r.palm_biometricsRepo.GetPalm_biometricsById(ctx, id)
	if err != nil {
		return palm_biometrics, errors.New(app.MsgRepositoryError)
	}

	return palm_biometrics, nil
}
func (r Resource)  GetPalm_biometricsByBiometricCode(ctx context.Context, biometriccode string) (palm_biometrics *model.Palm_biometrics, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Palm_biometricsService -> GetPalm_biometricsByBiometricCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetPalm_biometricsByBiometricCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetPalm_biometricsByBiometricCode.biometriccode", biometriccode)
	palm_biometrics, err = r.palm_biometricsRepo.GetPalm_biometricsByBiometricCode(ctx, biometriccode)
	if err != nil {
		return palm_biometrics, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetPalm_biometricsByBiometricCode.found", palm_biometrics != nil)
	return palm_biometrics, nil
}
func (r Resource)  DeletePalm_biometricsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Palm_biometricsService -> DeletePalm_biometricsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeletePalm_biometricsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeletePalm_biometricsById.id",id)

	result, err = r.palm_biometricsRepo.DeletePalm_biometricsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeletePalm_biometricsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertPalm_biometrics(ctx context.Context,palm_biometrics *model.Palm_biometrics) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Palm_biometricsService -> InsertPalm_biometrics", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertPalm_biometrics")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertPalm_biometrics.biometriccode", palm_biometrics.BiometricCode)
	insertedId, err = r.palm_biometricsRepo.InsertPalm_biometrics(ctx, palm_biometrics)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertPalm_biometrics.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdatePalm_biometrics(ctx context.Context,palm_biometrics *model.Palm_biometrics, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Palm_biometricsService -> UpdatePalm_biometrics", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdatePalm_biometrics")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdatePalm_biometrics.id", id)
	tracker.AddParam("service.UpdatePalm_biometrics.biometriccode", palm_biometrics.BiometricCode)

	err = r.palm_biometricsRepo.UpdatePalm_biometrics(ctx, palm_biometrics, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdatePalm_biometrics.updated", true)

	return nil
}

