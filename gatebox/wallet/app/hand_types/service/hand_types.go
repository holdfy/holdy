package hand_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	hand_typesRepo "palm-pay/app/hand_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Hand_typesServiceIF interface {
     GetHand_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetHand_typesById(ctx context.Context, id int64) (*model.Hand_types, error)
     GetHand_typesByTypeCode(ctx context.Context, typecode string) (*model.Hand_types, error)
     InsertHand_types(ctx context.Context, hand_types *model.Hand_types) (int64, error)
     UpdateHand_types(ctx context.Context, hand_types *model.Hand_types, id int64) error
     DeleteHand_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     hand_typesRepo hand_typesRepo.Hand_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewHand_typesService(hand_typesRepo hand_typesRepo.Hand_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         hand_typesRepo: hand_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.hand_types"),
     }
}
func (r Resource)  GetHand_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Hand_typesService -> GetHand_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetHand_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetHand_types.offset", offset)
	tracker.AddParam("service.GetHand_types.limit", limit)



	itemsPage, err = r.hand_typesRepo.GetHand_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Hand_types); ok {
		tracker.AddResult("service.GetHand_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetHand_typesById(ctx context.Context, id int64) (hand_types *model.Hand_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Hand_typesService -> GetHand_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetHand_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetHand_typesById.id", id)
	hand_types, err = r.hand_typesRepo.GetHand_typesById(ctx, id)
	if err != nil {
		return hand_types, errors.New(app.MsgRepositoryError)
	}

	return hand_types, nil
}
func (r Resource)  GetHand_typesByTypeCode(ctx context.Context, typecode string) (hand_types *model.Hand_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Hand_typesService -> GetHand_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetHand_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetHand_typesByTypeCode.typecode", typecode)
	hand_types, err = r.hand_typesRepo.GetHand_typesByTypeCode(ctx, typecode)
	if err != nil {
		return hand_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetHand_typesByTypeCode.found", hand_types != nil)
	return hand_types, nil
}
func (r Resource)  DeleteHand_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Hand_typesService -> DeleteHand_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteHand_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteHand_typesById.id",id)

	result, err = r.hand_typesRepo.DeleteHand_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteHand_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertHand_types(ctx context.Context,hand_types *model.Hand_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Hand_typesService -> InsertHand_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertHand_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertHand_types.typecode", hand_types.TypeCode)
	insertedId, err = r.hand_typesRepo.InsertHand_types(ctx, hand_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertHand_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateHand_types(ctx context.Context,hand_types *model.Hand_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Hand_typesService -> UpdateHand_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateHand_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateHand_types.id", id)
	tracker.AddParam("service.UpdateHand_types.typecode", hand_types.TypeCode)

	err = r.hand_typesRepo.UpdateHand_types(ctx, hand_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateHand_types.updated", true)

	return nil
}

