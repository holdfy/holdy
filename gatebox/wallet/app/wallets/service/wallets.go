package walletsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	walletsRepo "palm-pay/app/wallets/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type WalletsServiceIF interface {
     GetWallets(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetWalletsById(ctx context.Context, id int64) (*model.Wallets, error)
     GetWalletsByWalletCode(ctx context.Context, walletcode string) (*model.Wallets, error)
     InsertWallets(ctx context.Context, wallets *model.Wallets) (int64, error)
     UpdateWallets(ctx context.Context, wallets *model.Wallets, id int64) error
     DeleteWalletsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     walletsRepo walletsRepo.WalletsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewWalletsService(walletsRepo walletsRepo.WalletsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         walletsRepo: walletsRepo,
		  observability:  observabilidade.NewServiceObservability("service.wallets"),
     }
}
func (r Resource)  GetWallets(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("WalletsService -> GetWallets", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetWallets.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetWallets.offset", offset)
	tracker.AddParam("service.GetWallets.limit", limit)



	itemsPage, err = r.walletsRepo.GetWallets(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Wallets); ok {
		tracker.AddResult("service.GetWallets.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetWalletsById(ctx context.Context, id int64) (wallets *model.Wallets, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("WalletsService -> GetWalletsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetWalletsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetWalletsById.id", id)
	wallets, err = r.walletsRepo.GetWalletsById(ctx, id)
	if err != nil {
		return wallets, errors.New(app.MsgRepositoryError)
	}

	return wallets, nil
}
func (r Resource)  GetWalletsByWalletCode(ctx context.Context, walletcode string) (wallets *model.Wallets, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("WalletsService -> GetWalletsByWalletCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetWalletsByWalletCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetWalletsByWalletCode.walletcode", walletcode)
	wallets, err = r.walletsRepo.GetWalletsByWalletCode(ctx, walletcode)
	if err != nil {
		return wallets, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetWalletsByWalletCode.found", wallets != nil)
	return wallets, nil
}
func (r Resource)  DeleteWalletsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("WalletsService -> DeleteWalletsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteWalletsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteWalletsById.id",id)

	result, err = r.walletsRepo.DeleteWalletsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteWalletsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertWallets(ctx context.Context,wallets *model.Wallets) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("WalletsService -> InsertWallets", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertWallets")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertWallets.walletcode", wallets.WalletCode)
	insertedId, err = r.walletsRepo.InsertWallets(ctx, wallets)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertWallets.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateWallets(ctx context.Context,wallets *model.Wallets, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("WalletsService -> UpdateWallets", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateWallets")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateWallets.id", id)
	tracker.AddParam("service.UpdateWallets.walletcode", wallets.WalletCode)

	err = r.walletsRepo.UpdateWallets(ctx, wallets, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateWallets.updated", true)

	return nil
}

