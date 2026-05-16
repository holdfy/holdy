package security_event_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	security_event_typesRepo "palm-pay/app/security_event_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Security_event_typesServiceIF interface {
     GetSecurity_event_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetSecurity_event_typesById(ctx context.Context, id int64) (*model.Security_event_types, error)
     GetSecurity_event_typesByTypeCode(ctx context.Context, typecode string) (*model.Security_event_types, error)
     InsertSecurity_event_types(ctx context.Context, security_event_types *model.Security_event_types) (int64, error)
     UpdateSecurity_event_types(ctx context.Context, security_event_types *model.Security_event_types, id int64) error
     DeleteSecurity_event_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     security_event_typesRepo security_event_typesRepo.Security_event_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewSecurity_event_typesService(security_event_typesRepo security_event_typesRepo.Security_event_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         security_event_typesRepo: security_event_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.security_event_types"),
     }
}
func (r Resource)  GetSecurity_event_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_event_typesService -> GetSecurity_event_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSecurity_event_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetSecurity_event_types.offset", offset)
	tracker.AddParam("service.GetSecurity_event_types.limit", limit)



	itemsPage, err = r.security_event_typesRepo.GetSecurity_event_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Security_event_types); ok {
		tracker.AddResult("service.GetSecurity_event_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetSecurity_event_typesById(ctx context.Context, id int64) (security_event_types *model.Security_event_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_event_typesService -> GetSecurity_event_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetSecurity_event_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetSecurity_event_typesById.id", id)
	security_event_types, err = r.security_event_typesRepo.GetSecurity_event_typesById(ctx, id)
	if err != nil {
		return security_event_types, errors.New(app.MsgRepositoryError)
	}

	return security_event_types, nil
}
func (r Resource)  GetSecurity_event_typesByTypeCode(ctx context.Context, typecode string) (security_event_types *model.Security_event_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_event_typesService -> GetSecurity_event_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetSecurity_event_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetSecurity_event_typesByTypeCode.typecode", typecode)
	security_event_types, err = r.security_event_typesRepo.GetSecurity_event_typesByTypeCode(ctx, typecode)
	if err != nil {
		return security_event_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetSecurity_event_typesByTypeCode.found", security_event_types != nil)
	return security_event_types, nil
}
func (r Resource)  DeleteSecurity_event_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_event_typesService -> DeleteSecurity_event_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteSecurity_event_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteSecurity_event_typesById.id",id)

	result, err = r.security_event_typesRepo.DeleteSecurity_event_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteSecurity_event_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertSecurity_event_types(ctx context.Context,security_event_types *model.Security_event_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_event_typesService -> InsertSecurity_event_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertSecurity_event_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertSecurity_event_types.typecode", security_event_types.TypeCode)
	insertedId, err = r.security_event_typesRepo.InsertSecurity_event_types(ctx, security_event_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertSecurity_event_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateSecurity_event_types(ctx context.Context,security_event_types *model.Security_event_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Security_event_typesService -> UpdateSecurity_event_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateSecurity_event_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateSecurity_event_types.id", id)
	tracker.AddParam("service.UpdateSecurity_event_types.typecode", security_event_types.TypeCode)

	err = r.security_event_typesRepo.UpdateSecurity_event_types(ctx, security_event_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateSecurity_event_types.updated", true)

	return nil
}

