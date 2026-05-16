package device_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	device_typesRepo "palm-pay/app/device_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Device_typesServiceIF interface {
     GetDevice_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetDevice_typesById(ctx context.Context, id int64) (*model.Device_types, error)
     GetDevice_typesByTypeCode(ctx context.Context, typecode string) (*model.Device_types, error)
     InsertDevice_types(ctx context.Context, device_types *model.Device_types) (int64, error)
     UpdateDevice_types(ctx context.Context, device_types *model.Device_types, id int64) error
     DeleteDevice_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     device_typesRepo device_typesRepo.Device_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewDevice_typesService(device_typesRepo device_typesRepo.Device_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         device_typesRepo: device_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.device_types"),
     }
}
func (r Resource)  GetDevice_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Device_typesService -> GetDevice_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetDevice_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetDevice_types.offset", offset)
	tracker.AddParam("service.GetDevice_types.limit", limit)



	itemsPage, err = r.device_typesRepo.GetDevice_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Device_types); ok {
		tracker.AddResult("service.GetDevice_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetDevice_typesById(ctx context.Context, id int64) (device_types *model.Device_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Device_typesService -> GetDevice_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetDevice_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetDevice_typesById.id", id)
	device_types, err = r.device_typesRepo.GetDevice_typesById(ctx, id)
	if err != nil {
		return device_types, errors.New(app.MsgRepositoryError)
	}

	return device_types, nil
}
func (r Resource)  GetDevice_typesByTypeCode(ctx context.Context, typecode string) (device_types *model.Device_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Device_typesService -> GetDevice_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetDevice_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetDevice_typesByTypeCode.typecode", typecode)
	device_types, err = r.device_typesRepo.GetDevice_typesByTypeCode(ctx, typecode)
	if err != nil {
		return device_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetDevice_typesByTypeCode.found", device_types != nil)
	return device_types, nil
}
func (r Resource)  DeleteDevice_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Device_typesService -> DeleteDevice_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteDevice_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteDevice_typesById.id",id)

	result, err = r.device_typesRepo.DeleteDevice_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteDevice_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertDevice_types(ctx context.Context,device_types *model.Device_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Device_typesService -> InsertDevice_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertDevice_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertDevice_types.typecode", device_types.TypeCode)
	insertedId, err = r.device_typesRepo.InsertDevice_types(ctx, device_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertDevice_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateDevice_types(ctx context.Context,device_types *model.Device_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Device_typesService -> UpdateDevice_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateDevice_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateDevice_types.id", id)
	tracker.AddParam("service.UpdateDevice_types.typecode", device_types.TypeCode)

	err = r.device_typesRepo.UpdateDevice_types(ctx, device_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateDevice_types.updated", true)

	return nil
}

