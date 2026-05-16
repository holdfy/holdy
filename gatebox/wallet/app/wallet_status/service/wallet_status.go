package wallet_statusSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	wallet_statusRepo "palm-pay/app/wallet_status/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Wallet_statusServiceIF interface {
     GetWallet_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetWallet_statusById(ctx context.Context, id int64) (*model.Wallet_status, error)
     GetWallet_statusByStatusCode(ctx context.Context, statuscode string) (*model.Wallet_status, error)
     InsertWallet_status(ctx context.Context, wallet_status *model.Wallet_status) (int64, error)
     UpdateWallet_status(ctx context.Context, wallet_status *model.Wallet_status, id int64) error
     DeleteWallet_statusById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     wallet_statusRepo wallet_statusRepo.Wallet_statusRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewWallet_statusService(wallet_statusRepo wallet_statusRepo.Wallet_statusRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         wallet_statusRepo: wallet_statusRepo,
		  observability:  observabilidade.NewServiceObservability("service.wallet_status"),
     }
}
func (r Resource)  GetWallet_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_statusService -> GetWallet_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetWallet_status.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetWallet_status.offset", offset)
	tracker.AddParam("service.GetWallet_status.limit", limit)



	itemsPage, err = r.wallet_statusRepo.GetWallet_status(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Wallet_status); ok {
		tracker.AddResult("service.GetWallet_status.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetWallet_statusById(ctx context.Context, id int64) (wallet_status *model.Wallet_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_statusService -> GetWallet_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetWallet_statusById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetWallet_statusById.id", id)
	wallet_status, err = r.wallet_statusRepo.GetWallet_statusById(ctx, id)
	if err != nil {
		return wallet_status, errors.New(app.MsgRepositoryError)
	}

	return wallet_status, nil
}
func (r Resource)  GetWallet_statusByStatusCode(ctx context.Context, statuscode string) (wallet_status *model.Wallet_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_statusService -> GetWallet_statusByStatusCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetWallet_statusByStatusCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetWallet_statusByStatusCode.statuscode", statuscode)
	wallet_status, err = r.wallet_statusRepo.GetWallet_statusByStatusCode(ctx, statuscode)
	if err != nil {
		return wallet_status, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetWallet_statusByStatusCode.found", wallet_status != nil)
	return wallet_status, nil
}
func (r Resource)  DeleteWallet_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_statusService -> DeleteWallet_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteWallet_statusById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteWallet_statusById.id",id)

	result, err = r.wallet_statusRepo.DeleteWallet_statusById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteWallet_statusById.deleted", result)
	return result, nil
}
func (r Resource)  InsertWallet_status(ctx context.Context,wallet_status *model.Wallet_status) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_statusService -> InsertWallet_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertWallet_status")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertWallet_status.statuscode", wallet_status.StatusCode)
	insertedId, err = r.wallet_statusRepo.InsertWallet_status(ctx, wallet_status)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertWallet_status.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateWallet_status(ctx context.Context,wallet_status *model.Wallet_status, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_statusService -> UpdateWallet_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateWallet_status")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateWallet_status.id", id)
	tracker.AddParam("service.UpdateWallet_status.statuscode", wallet_status.StatusCode)

	err = r.wallet_statusRepo.UpdateWallet_status(ctx, wallet_status, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateWallet_status.updated", true)

	return nil
}

