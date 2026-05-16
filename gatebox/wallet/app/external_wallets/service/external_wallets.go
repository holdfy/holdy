package external_walletsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	external_walletsRepo "palm-pay/app/external_wallets/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type External_walletsServiceIF interface {
     GetExternal_wallets(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetExternal_walletsById(ctx context.Context, id int64) (*model.External_wallets, error)
     GetExternal_walletsByExternalWalletCode(ctx context.Context, externalwalletcode string) (*model.External_wallets, error)
     InsertExternal_wallets(ctx context.Context, external_wallets *model.External_wallets) (int64, error)
     UpdateExternal_wallets(ctx context.Context, external_wallets *model.External_wallets, id int64) error
     DeleteExternal_walletsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     external_walletsRepo external_walletsRepo.External_walletsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewExternal_walletsService(external_walletsRepo external_walletsRepo.External_walletsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         external_walletsRepo: external_walletsRepo,
		  observability:  observabilidade.NewServiceObservability("service.external_wallets"),
     }
}
func (r Resource)  GetExternal_wallets(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("External_walletsService -> GetExternal_wallets", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetExternal_wallets.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetExternal_wallets.offset", offset)
	tracker.AddParam("service.GetExternal_wallets.limit", limit)



	itemsPage, err = r.external_walletsRepo.GetExternal_wallets(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.External_wallets); ok {
		tracker.AddResult("service.GetExternal_wallets.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetExternal_walletsById(ctx context.Context, id int64) (external_wallets *model.External_wallets, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("External_walletsService -> GetExternal_walletsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetExternal_walletsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetExternal_walletsById.id", id)
	external_wallets, err = r.external_walletsRepo.GetExternal_walletsById(ctx, id)
	if err != nil {
		return external_wallets, errors.New(app.MsgRepositoryError)
	}

	return external_wallets, nil
}
func (r Resource)  GetExternal_walletsByExternalWalletCode(ctx context.Context, externalwalletcode string) (external_wallets *model.External_wallets, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("External_walletsService -> GetExternal_walletsByExternalWalletCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetExternal_walletsByExternalWalletCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetExternal_walletsByExternalWalletCode.externalwalletcode", externalwalletcode)
	external_wallets, err = r.external_walletsRepo.GetExternal_walletsByExternalWalletCode(ctx, externalwalletcode)
	if err != nil {
		return external_wallets, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetExternal_walletsByExternalWalletCode.found", external_wallets != nil)
	return external_wallets, nil
}
func (r Resource)  DeleteExternal_walletsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("External_walletsService -> DeleteExternal_walletsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteExternal_walletsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteExternal_walletsById.id",id)

	result, err = r.external_walletsRepo.DeleteExternal_walletsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteExternal_walletsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertExternal_wallets(ctx context.Context,external_wallets *model.External_wallets) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("External_walletsService -> InsertExternal_wallets", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertExternal_wallets")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertExternal_wallets.externalwalletcode", external_wallets.ExternalWalletCode)
	insertedId, err = r.external_walletsRepo.InsertExternal_wallets(ctx, external_wallets)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertExternal_wallets.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateExternal_wallets(ctx context.Context,external_wallets *model.External_wallets, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("External_walletsService -> UpdateExternal_wallets", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateExternal_wallets")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateExternal_wallets.id", id)
	tracker.AddParam("service.UpdateExternal_wallets.externalwalletcode", external_wallets.ExternalWalletCode)

	err = r.external_walletsRepo.UpdateExternal_wallets(ctx, external_wallets, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateExternal_wallets.updated", true)

	return nil
}

