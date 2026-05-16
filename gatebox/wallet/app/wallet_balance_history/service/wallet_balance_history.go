package wallet_balance_historySV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	wallet_balance_historyRepo "palm-pay/app/wallet_balance_history/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Wallet_balance_historyServiceIF interface {
     GetWallet_balance_history(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetWallet_balance_historyById(ctx context.Context, id int64) (*model.Wallet_balance_history, error)
     GetWallet_balance_historyByBalanceHistoryCode(ctx context.Context, balancehistorycode string) (*model.Wallet_balance_history, error)
     InsertWallet_balance_history(ctx context.Context, wallet_balance_history *model.Wallet_balance_history) (int64, error)
     UpdateWallet_balance_history(ctx context.Context, wallet_balance_history *model.Wallet_balance_history, id int64) error
     DeleteWallet_balance_historyById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     wallet_balance_historyRepo wallet_balance_historyRepo.Wallet_balance_historyRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewWallet_balance_historyService(wallet_balance_historyRepo wallet_balance_historyRepo.Wallet_balance_historyRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         wallet_balance_historyRepo: wallet_balance_historyRepo,
		  observability:  observabilidade.NewServiceObservability("service.wallet_balance_history"),
     }
}
func (r Resource)  GetWallet_balance_history(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_balance_historyService -> GetWallet_balance_history", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetWallet_balance_history.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetWallet_balance_history.offset", offset)
	tracker.AddParam("service.GetWallet_balance_history.limit", limit)



	itemsPage, err = r.wallet_balance_historyRepo.GetWallet_balance_history(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Wallet_balance_history); ok {
		tracker.AddResult("service.GetWallet_balance_history.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetWallet_balance_historyById(ctx context.Context, id int64) (wallet_balance_history *model.Wallet_balance_history, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_balance_historyService -> GetWallet_balance_historyById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetWallet_balance_historyById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetWallet_balance_historyById.id", id)
	wallet_balance_history, err = r.wallet_balance_historyRepo.GetWallet_balance_historyById(ctx, id)
	if err != nil {
		return wallet_balance_history, errors.New(app.MsgRepositoryError)
	}

	return wallet_balance_history, nil
}
func (r Resource)  GetWallet_balance_historyByBalanceHistoryCode(ctx context.Context, balancehistorycode string) (wallet_balance_history *model.Wallet_balance_history, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_balance_historyService -> GetWallet_balance_historyByBalanceHistoryCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetWallet_balance_historyByBalanceHistoryCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetWallet_balance_historyByBalanceHistoryCode.balancehistorycode", balancehistorycode)
	wallet_balance_history, err = r.wallet_balance_historyRepo.GetWallet_balance_historyByBalanceHistoryCode(ctx, balancehistorycode)
	if err != nil {
		return wallet_balance_history, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetWallet_balance_historyByBalanceHistoryCode.found", wallet_balance_history != nil)
	return wallet_balance_history, nil
}
func (r Resource)  DeleteWallet_balance_historyById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_balance_historyService -> DeleteWallet_balance_historyById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteWallet_balance_historyById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteWallet_balance_historyById.id",id)

	result, err = r.wallet_balance_historyRepo.DeleteWallet_balance_historyById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteWallet_balance_historyById.deleted", result)
	return result, nil
}
func (r Resource)  InsertWallet_balance_history(ctx context.Context,wallet_balance_history *model.Wallet_balance_history) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_balance_historyService -> InsertWallet_balance_history", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertWallet_balance_history")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertWallet_balance_history.balancehistorycode", wallet_balance_history.BalanceHistoryCode)
	insertedId, err = r.wallet_balance_historyRepo.InsertWallet_balance_history(ctx, wallet_balance_history)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertWallet_balance_history.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateWallet_balance_history(ctx context.Context,wallet_balance_history *model.Wallet_balance_history, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_balance_historyService -> UpdateWallet_balance_history", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateWallet_balance_history")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateWallet_balance_history.id", id)
	tracker.AddParam("service.UpdateWallet_balance_history.balancehistorycode", wallet_balance_history.BalanceHistoryCode)

	err = r.wallet_balance_historyRepo.UpdateWallet_balance_history(ctx, wallet_balance_history, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateWallet_balance_history.updated", true)

	return nil
}

