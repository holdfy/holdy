package wallet_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	wallet_typesRepo "palm-pay/app/wallet_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Wallet_typesServiceIF interface {
     GetWallet_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetWallet_typesById(ctx context.Context, id int64) (*model.Wallet_types, error)
     GetWallet_typesByTypeCode(ctx context.Context, typecode string) (*model.Wallet_types, error)
     InsertWallet_types(ctx context.Context, wallet_types *model.Wallet_types) (int64, error)
     UpdateWallet_types(ctx context.Context, wallet_types *model.Wallet_types, id int64) error
     DeleteWallet_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     wallet_typesRepo wallet_typesRepo.Wallet_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewWallet_typesService(wallet_typesRepo wallet_typesRepo.Wallet_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         wallet_typesRepo: wallet_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.wallet_types"),
     }
}
func (r Resource)  GetWallet_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_typesService -> GetWallet_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetWallet_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetWallet_types.offset", offset)
	tracker.AddParam("service.GetWallet_types.limit", limit)



	itemsPage, err = r.wallet_typesRepo.GetWallet_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Wallet_types); ok {
		tracker.AddResult("service.GetWallet_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetWallet_typesById(ctx context.Context, id int64) (wallet_types *model.Wallet_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_typesService -> GetWallet_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetWallet_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetWallet_typesById.id", id)
	wallet_types, err = r.wallet_typesRepo.GetWallet_typesById(ctx, id)
	if err != nil {
		return wallet_types, errors.New(app.MsgRepositoryError)
	}

	return wallet_types, nil
}
func (r Resource)  GetWallet_typesByTypeCode(ctx context.Context, typecode string) (wallet_types *model.Wallet_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_typesService -> GetWallet_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetWallet_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetWallet_typesByTypeCode.typecode", typecode)
	wallet_types, err = r.wallet_typesRepo.GetWallet_typesByTypeCode(ctx, typecode)
	if err != nil {
		return wallet_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetWallet_typesByTypeCode.found", wallet_types != nil)
	return wallet_types, nil
}
func (r Resource)  DeleteWallet_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_typesService -> DeleteWallet_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteWallet_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteWallet_typesById.id",id)

	result, err = r.wallet_typesRepo.DeleteWallet_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteWallet_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertWallet_types(ctx context.Context,wallet_types *model.Wallet_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_typesService -> InsertWallet_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertWallet_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertWallet_types.typecode", wallet_types.TypeCode)
	insertedId, err = r.wallet_typesRepo.InsertWallet_types(ctx, wallet_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertWallet_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateWallet_types(ctx context.Context,wallet_types *model.Wallet_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Wallet_typesService -> UpdateWallet_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateWallet_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateWallet_types.id", id)
	tracker.AddParam("service.UpdateWallet_types.typecode", wallet_types.TypeCode)

	err = r.wallet_typesRepo.UpdateWallet_types(ctx, wallet_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateWallet_types.updated", true)

	return nil
}

