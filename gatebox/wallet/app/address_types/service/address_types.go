package address_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	address_typesRepo "palm-pay/app/address_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Address_typesServiceIF interface {
     GetAddress_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetAddress_typesById(ctx context.Context, id int64) (*model.Address_types, error)
     GetAddress_typesByTypeCode(ctx context.Context, typecode string) (*model.Address_types, error)
     InsertAddress_types(ctx context.Context, address_types *model.Address_types) (int64, error)
     UpdateAddress_types(ctx context.Context, address_types *model.Address_types, id int64) error
     DeleteAddress_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     address_typesRepo address_typesRepo.Address_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewAddress_typesService(address_typesRepo address_typesRepo.Address_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         address_typesRepo: address_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.address_types"),
     }
}
func (r Resource)  GetAddress_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Address_typesService -> GetAddress_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAddress_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetAddress_types.offset", offset)
	tracker.AddParam("service.GetAddress_types.limit", limit)



	itemsPage, err = r.address_typesRepo.GetAddress_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Address_types); ok {
		tracker.AddResult("service.GetAddress_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetAddress_typesById(ctx context.Context, id int64) (address_types *model.Address_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Address_typesService -> GetAddress_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetAddress_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetAddress_typesById.id", id)
	address_types, err = r.address_typesRepo.GetAddress_typesById(ctx, id)
	if err != nil {
		return address_types, errors.New(app.MsgRepositoryError)
	}

	return address_types, nil
}
func (r Resource)  GetAddress_typesByTypeCode(ctx context.Context, typecode string) (address_types *model.Address_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Address_typesService -> GetAddress_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetAddress_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetAddress_typesByTypeCode.typecode", typecode)
	address_types, err = r.address_typesRepo.GetAddress_typesByTypeCode(ctx, typecode)
	if err != nil {
		return address_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetAddress_typesByTypeCode.found", address_types != nil)
	return address_types, nil
}
func (r Resource)  DeleteAddress_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Address_typesService -> DeleteAddress_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteAddress_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteAddress_typesById.id",id)

	result, err = r.address_typesRepo.DeleteAddress_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteAddress_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertAddress_types(ctx context.Context,address_types *model.Address_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Address_typesService -> InsertAddress_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertAddress_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertAddress_types.typecode", address_types.TypeCode)
	insertedId, err = r.address_typesRepo.InsertAddress_types(ctx, address_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertAddress_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateAddress_types(ctx context.Context,address_types *model.Address_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Address_typesService -> UpdateAddress_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateAddress_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateAddress_types.id", id)
	tracker.AddParam("service.UpdateAddress_types.typecode", address_types.TypeCode)

	err = r.address_typesRepo.UpdateAddress_types(ctx, address_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateAddress_types.updated", true)

	return nil
}

