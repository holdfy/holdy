package account_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	account_typesRepo "palm-pay/app/account_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Account_typesServiceIF interface {
     GetAccount_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAccount_typesById(ctx context.Context, id int64) (*model.Account_types, error)
     GetAccount_typesByTypeCode(ctx context.Context, typecode string) (*model.Account_types, error)
     InsertAccount_types(ctx context.Context, account_types *model.Account_types) (int64, error)
     UpdateAccount_types(ctx context.Context, account_types *model.Account_types, id int64) error
     DeleteAccount_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     account_typesRepo account_typesRepo.Account_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewAccount_typesService(account_typesRepo account_typesRepo.Account_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         account_typesRepo: account_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.account_types"),
     }
}
func (r Resource)  GetAccount_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Account_typesService -> GetAccount_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAccount_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetAccount_types.offset", offset)
	tracker.AddParam("service.GetAccount_types.limit", limit)



	itemsPage, err = r.account_typesRepo.GetAccount_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Account_types); ok {
		tracker.AddResult("service.GetAccount_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetAccount_typesById(ctx context.Context, id int64) (account_types *model.Account_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Account_typesService -> GetAccount_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAccount_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetAccount_typesById.id", id)
	account_types, err = r.account_typesRepo.GetAccount_typesById(ctx, id)
	if err != nil {
		return account_types, errors.New(app.MsgRepositoryError)
	}

	return account_types, nil
}
func (r Resource)  GetAccount_typesByTypeCode(ctx context.Context, typecode string) (account_types *model.Account_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Account_typesService -> GetAccount_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetAccount_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetAccount_typesByTypeCode.typecode", typecode)
	account_types, err = r.account_typesRepo.GetAccount_typesByTypeCode(ctx, typecode)
	if err != nil {
		return account_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetAccount_typesByTypeCode.found", account_types != nil)
	return account_types, nil
}
func (r Resource)  DeleteAccount_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Account_typesService -> DeleteAccount_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteAccount_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteAccount_typesById.id",id)

	result, err = r.account_typesRepo.DeleteAccount_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteAccount_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertAccount_types(ctx context.Context,account_types *model.Account_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Account_typesService -> InsertAccount_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertAccount_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertAccount_types.typecode", account_types.TypeCode)
	insertedId, err = r.account_typesRepo.InsertAccount_types(ctx, account_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertAccount_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateAccount_types(ctx context.Context,account_types *model.Account_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Account_typesService -> UpdateAccount_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateAccount_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateAccount_types.id", id)
	tracker.AddParam("service.UpdateAccount_types.typecode", account_types.TypeCode)

	err = r.account_typesRepo.UpdateAccount_types(ctx, account_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateAccount_types.updated", true)

	return nil
}

