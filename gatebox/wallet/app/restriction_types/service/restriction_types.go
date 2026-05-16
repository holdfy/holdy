package restriction_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	restriction_typesRepo "palm-pay/app/restriction_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Restriction_typesServiceIF interface {
     GetRestriction_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetRestriction_typesById(ctx context.Context, id int64) (*model.Restriction_types, error)
     GetRestriction_typesByTypeCode(ctx context.Context, typecode string) (*model.Restriction_types, error)
     InsertRestriction_types(ctx context.Context, restriction_types *model.Restriction_types) (int64, error)
     UpdateRestriction_types(ctx context.Context, restriction_types *model.Restriction_types, id int64) error
     DeleteRestriction_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     restriction_typesRepo restriction_typesRepo.Restriction_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewRestriction_typesService(restriction_typesRepo restriction_typesRepo.Restriction_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         restriction_typesRepo: restriction_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.restriction_types"),
     }
}
func (r Resource)  GetRestriction_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Restriction_typesService -> GetRestriction_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetRestriction_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetRestriction_types.offset", offset)
	tracker.AddParam("service.GetRestriction_types.limit", limit)



	itemsPage, err = r.restriction_typesRepo.GetRestriction_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Restriction_types); ok {
		tracker.AddResult("service.GetRestriction_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetRestriction_typesById(ctx context.Context, id int64) (restriction_types *model.Restriction_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Restriction_typesService -> GetRestriction_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetRestriction_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetRestriction_typesById.id", id)
	restriction_types, err = r.restriction_typesRepo.GetRestriction_typesById(ctx, id)
	if err != nil {
		return restriction_types, errors.New(app.MsgRepositoryError)
	}

	return restriction_types, nil
}
func (r Resource)  GetRestriction_typesByTypeCode(ctx context.Context, typecode string) (restriction_types *model.Restriction_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Restriction_typesService -> GetRestriction_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetRestriction_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetRestriction_typesByTypeCode.typecode", typecode)
	restriction_types, err = r.restriction_typesRepo.GetRestriction_typesByTypeCode(ctx, typecode)
	if err != nil {
		return restriction_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetRestriction_typesByTypeCode.found", restriction_types != nil)
	return restriction_types, nil
}
func (r Resource)  DeleteRestriction_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Restriction_typesService -> DeleteRestriction_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteRestriction_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteRestriction_typesById.id",id)

	result, err = r.restriction_typesRepo.DeleteRestriction_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteRestriction_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertRestriction_types(ctx context.Context,restriction_types *model.Restriction_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Restriction_typesService -> InsertRestriction_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertRestriction_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertRestriction_types.typecode", restriction_types.TypeCode)
	insertedId, err = r.restriction_typesRepo.InsertRestriction_types(ctx, restriction_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertRestriction_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateRestriction_types(ctx context.Context,restriction_types *model.Restriction_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Restriction_typesService -> UpdateRestriction_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateRestriction_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateRestriction_types.id", id)
	tracker.AddParam("service.UpdateRestriction_types.typecode", restriction_types.TypeCode)

	err = r.restriction_typesRepo.UpdateRestriction_types(ctx, restriction_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateRestriction_types.updated", true)

	return nil
}

