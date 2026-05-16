package wallet_providersSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	wallet_providersRepo "palm-pay/app/wallet_providers/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Wallet_providersServiceIF interface {
     GetWallet_providers(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetWallet_providersById(ctx context.Context, id int64) (*model.Wallet_providers, error)
     GetWallet_providersByProviderCode(ctx context.Context, providercode string) (*model.Wallet_providers, error)
     InsertWallet_providers(ctx context.Context, wallet_providers *model.Wallet_providers) (int64, error)
     UpdateWallet_providers(ctx context.Context, wallet_providers *model.Wallet_providers, id int64) error
     DeleteWallet_providersById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     wallet_providersRepo wallet_providersRepo.Wallet_providersRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewWallet_providersService(wallet_providersRepo wallet_providersRepo.Wallet_providersRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         wallet_providersRepo: wallet_providersRepo,
		  observability:  observabilidade.NewServiceObservability("service.wallet_providers"),
     }
}
func (r Resource)  GetWallet_providers(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_providersService -> GetWallet_providers", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetWallet_providers.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetWallet_providers.offset", offset)
	tracker.AddParam("service.GetWallet_providers.limit", limit)



	itemsPage, err = r.wallet_providersRepo.GetWallet_providers(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Wallet_providers); ok {
		tracker.AddResult("service.GetWallet_providers.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetWallet_providersById(ctx context.Context, id int64) (wallet_providers *model.Wallet_providers, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_providersService -> GetWallet_providersById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetWallet_providersById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetWallet_providersById.id", id)
	wallet_providers, err = r.wallet_providersRepo.GetWallet_providersById(ctx, id)
	if err != nil {
		return wallet_providers, errors.New(app.MsgRepositoryError)
	}

	return wallet_providers, nil
}
func (r Resource)  GetWallet_providersByProviderCode(ctx context.Context, providercode string) (wallet_providers *model.Wallet_providers, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_providersService -> GetWallet_providersByProviderCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetWallet_providersByProviderCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetWallet_providersByProviderCode.providercode", providercode)
	wallet_providers, err = r.wallet_providersRepo.GetWallet_providersByProviderCode(ctx, providercode)
	if err != nil {
		return wallet_providers, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetWallet_providersByProviderCode.found", wallet_providers != nil)
	return wallet_providers, nil
}
func (r Resource)  DeleteWallet_providersById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_providersService -> DeleteWallet_providersById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteWallet_providersById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteWallet_providersById.id",id)

	result, err = r.wallet_providersRepo.DeleteWallet_providersById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteWallet_providersById.deleted", result)
	return result, nil
}
func (r Resource)  InsertWallet_providers(ctx context.Context,wallet_providers *model.Wallet_providers) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_providersService -> InsertWallet_providers", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertWallet_providers")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertWallet_providers.providercode", wallet_providers.ProviderCode)
	insertedId, err = r.wallet_providersRepo.InsertWallet_providers(ctx, wallet_providers)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertWallet_providers.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateWallet_providers(ctx context.Context,wallet_providers *model.Wallet_providers, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_providersService -> UpdateWallet_providers", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateWallet_providers")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateWallet_providers.id", id)
	tracker.AddParam("service.UpdateWallet_providers.providercode", wallet_providers.ProviderCode)

	err = r.wallet_providersRepo.UpdateWallet_providers(ctx, wallet_providers, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateWallet_providers.updated", true)

	return nil
}

