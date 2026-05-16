package transaction_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	transaction_typesRepo "palm-pay/app/transaction_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Transaction_typesServiceIF interface {
     GetTransaction_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetTransaction_typesById(ctx context.Context, id int64) (*model.Transaction_types, error)
     GetTransaction_typesByTypeCode(ctx context.Context, typecode string) (*model.Transaction_types, error)
     InsertTransaction_types(ctx context.Context, transaction_types *model.Transaction_types) (int64, error)
     UpdateTransaction_types(ctx context.Context, transaction_types *model.Transaction_types, id int64) error
     DeleteTransaction_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     transaction_typesRepo transaction_typesRepo.Transaction_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewTransaction_typesService(transaction_typesRepo transaction_typesRepo.Transaction_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         transaction_typesRepo: transaction_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.transaction_types"),
     }
}
func (r Resource)  GetTransaction_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_typesService -> GetTransaction_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetTransaction_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetTransaction_types.offset", offset)
	tracker.AddParam("service.GetTransaction_types.limit", limit)



	itemsPage, err = r.transaction_typesRepo.GetTransaction_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Transaction_types); ok {
		tracker.AddResult("service.GetTransaction_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetTransaction_typesById(ctx context.Context, id int64) (transaction_types *model.Transaction_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_typesService -> GetTransaction_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetTransaction_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetTransaction_typesById.id", id)
	transaction_types, err = r.transaction_typesRepo.GetTransaction_typesById(ctx, id)
	if err != nil {
		return transaction_types, errors.New(app.MsgRepositoryError)
	}

	return transaction_types, nil
}
func (r Resource)  GetTransaction_typesByTypeCode(ctx context.Context, typecode string) (transaction_types *model.Transaction_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_typesService -> GetTransaction_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetTransaction_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetTransaction_typesByTypeCode.typecode", typecode)
	transaction_types, err = r.transaction_typesRepo.GetTransaction_typesByTypeCode(ctx, typecode)
	if err != nil {
		return transaction_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetTransaction_typesByTypeCode.found", transaction_types != nil)
	return transaction_types, nil
}
func (r Resource)  DeleteTransaction_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_typesService -> DeleteTransaction_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteTransaction_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteTransaction_typesById.id",id)

	result, err = r.transaction_typesRepo.DeleteTransaction_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteTransaction_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertTransaction_types(ctx context.Context,transaction_types *model.Transaction_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_typesService -> InsertTransaction_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertTransaction_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertTransaction_types.typecode", transaction_types.TypeCode)
	insertedId, err = r.transaction_typesRepo.InsertTransaction_types(ctx, transaction_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertTransaction_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateTransaction_types(ctx context.Context,transaction_types *model.Transaction_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Transaction_typesService -> UpdateTransaction_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateTransaction_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateTransaction_types.id", id)
	tracker.AddParam("service.UpdateTransaction_types.typecode", transaction_types.TypeCode)

	err = r.transaction_typesRepo.UpdateTransaction_types(ctx, transaction_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateTransaction_types.updated", true)

	return nil
}

