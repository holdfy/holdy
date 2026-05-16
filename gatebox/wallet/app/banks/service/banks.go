package banksSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	banksRepo "palm-pay/app/banks/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type BanksServiceIF interface {
     GetBanks(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetBanksById(ctx context.Context, id int64) (*model.Banks, error)
     GetBanksByBankCodeInternal(ctx context.Context, bankcodeinternal string) (*model.Banks, error)
     InsertBanks(ctx context.Context, banks *model.Banks) (int64, error)
     UpdateBanks(ctx context.Context, banks *model.Banks, id int64) error
     DeleteBanksById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     banksRepo banksRepo.BanksRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewBanksService(banksRepo banksRepo.BanksRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         banksRepo: banksRepo,
		  observability:  observabilidade.NewServiceObservability("service.banks"),
     }
}
func (r Resource)  GetBanks(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("BanksService -> GetBanks", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetBanks.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetBanks.offset", offset)
	tracker.AddParam("service.GetBanks.limit", limit)



	itemsPage, err = r.banksRepo.GetBanks(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Banks); ok {
		tracker.AddResult("service.GetBanks.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetBanksById(ctx context.Context, id int64) (banks *model.Banks, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("BanksService -> GetBanksById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetBanksById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetBanksById.id", id)
	banks, err = r.banksRepo.GetBanksById(ctx, id)
	if err != nil {
		return banks, errors.New(app.MsgRepositoryError)
	}

	return banks, nil
}
func (r Resource)  GetBanksByBankCodeInternal(ctx context.Context, bankcodeinternal string) (banks *model.Banks, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("BanksService -> GetBanksByBankCodeInternal", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetBanksByBankCodeInternal")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetBanksByBankCodeInternal.bankcodeinternal", bankcodeinternal)
	banks, err = r.banksRepo.GetBanksByBankCodeInternal(ctx, bankcodeinternal)
	if err != nil {
		return banks, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetBanksByBankCodeInternal.found", banks != nil)
	return banks, nil
}
func (r Resource)  DeleteBanksById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("BanksService -> DeleteBanksById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteBanksById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteBanksById.id",id)

	result, err = r.banksRepo.DeleteBanksById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteBanksById.deleted", result)
	return result, nil
}
func (r Resource)  InsertBanks(ctx context.Context,banks *model.Banks) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("BanksService -> InsertBanks", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertBanks")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertBanks.bankcodeinternal", banks.BankCodeInternal)
	insertedId, err = r.banksRepo.InsertBanks(ctx, banks)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertBanks.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateBanks(ctx context.Context,banks *model.Banks, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("BanksService -> UpdateBanks", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateBanks")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateBanks.id", id)
	tracker.AddParam("service.UpdateBanks.bankcodeinternal", banks.BankCodeInternal)

	err = r.banksRepo.UpdateBanks(ctx, banks, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateBanks.updated", true)

	return nil
}

