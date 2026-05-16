package balance_change_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	balance_change_typesRepo "palm-pay/app/balance_change_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Balance_change_typesServiceIF interface {
     GetBalance_change_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetBalance_change_typesById(ctx context.Context, id int64) (*model.Balance_change_types, error)
     GetBalance_change_typesByTypeCode(ctx context.Context, typecode string) (*model.Balance_change_types, error)
     InsertBalance_change_types(ctx context.Context, balance_change_types *model.Balance_change_types) (int64, error)
     UpdateBalance_change_types(ctx context.Context, balance_change_types *model.Balance_change_types, id int64) error
     DeleteBalance_change_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     balance_change_typesRepo balance_change_typesRepo.Balance_change_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewBalance_change_typesService(balance_change_typesRepo balance_change_typesRepo.Balance_change_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         balance_change_typesRepo: balance_change_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.balance_change_types"),
     }
}
func (r Resource)  GetBalance_change_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Balance_change_typesService -> GetBalance_change_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetBalance_change_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetBalance_change_types.offset", offset)
	tracker.AddParam("service.GetBalance_change_types.limit", limit)



	itemsPage, err = r.balance_change_typesRepo.GetBalance_change_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Balance_change_types); ok {
		tracker.AddResult("service.GetBalance_change_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetBalance_change_typesById(ctx context.Context, id int64) (balance_change_types *model.Balance_change_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Balance_change_typesService -> GetBalance_change_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetBalance_change_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetBalance_change_typesById.id", id)
	balance_change_types, err = r.balance_change_typesRepo.GetBalance_change_typesById(ctx, id)
	if err != nil {
		return balance_change_types, errors.New(app.MsgRepositoryError)
	}

	return balance_change_types, nil
}
func (r Resource)  GetBalance_change_typesByTypeCode(ctx context.Context, typecode string) (balance_change_types *model.Balance_change_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Balance_change_typesService -> GetBalance_change_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetBalance_change_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetBalance_change_typesByTypeCode.typecode", typecode)
	balance_change_types, err = r.balance_change_typesRepo.GetBalance_change_typesByTypeCode(ctx, typecode)
	if err != nil {
		return balance_change_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetBalance_change_typesByTypeCode.found", balance_change_types != nil)
	return balance_change_types, nil
}
func (r Resource)  DeleteBalance_change_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Balance_change_typesService -> DeleteBalance_change_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteBalance_change_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteBalance_change_typesById.id",id)

	result, err = r.balance_change_typesRepo.DeleteBalance_change_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteBalance_change_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertBalance_change_types(ctx context.Context,balance_change_types *model.Balance_change_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Balance_change_typesService -> InsertBalance_change_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertBalance_change_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertBalance_change_types.typecode", balance_change_types.TypeCode)
	insertedId, err = r.balance_change_typesRepo.InsertBalance_change_types(ctx, balance_change_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertBalance_change_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateBalance_change_types(ctx context.Context,balance_change_types *model.Balance_change_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Balance_change_typesService -> UpdateBalance_change_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateBalance_change_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateBalance_change_types.id", id)
	tracker.AddParam("service.UpdateBalance_change_types.typecode", balance_change_types.TypeCode)

	err = r.balance_change_typesRepo.UpdateBalance_change_types(ctx, balance_change_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateBalance_change_types.updated", true)

	return nil
}

