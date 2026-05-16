package card_typesSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	card_typesRepo "palm-pay/app/card_types/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Card_typesServiceIF interface {
     GetCard_types(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetCard_typesById(ctx context.Context, id int64) (*model.Card_types, error)
     GetCard_typesByTypeCode(ctx context.Context, typecode string) (*model.Card_types, error)
     InsertCard_types(ctx context.Context, card_types *model.Card_types) (int64, error)
     UpdateCard_types(ctx context.Context, card_types *model.Card_types, id int64) error
     DeleteCard_typesById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     card_typesRepo card_typesRepo.Card_typesRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewCard_typesService(card_typesRepo card_typesRepo.Card_typesRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         card_typesRepo: card_typesRepo,
		  observability:  observabilidade.NewServiceObservability("service.card_types"),
     }
}
func (r Resource)  GetCard_types(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_typesService -> GetCard_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetCard_types.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetCard_types.offset", offset)
	tracker.AddParam("service.GetCard_types.limit", limit)



	itemsPage, err = r.card_typesRepo.GetCard_types(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Card_types); ok {
		tracker.AddResult("service.GetCard_types.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetCard_typesById(ctx context.Context, id int64) (card_types *model.Card_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_typesService -> GetCard_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetCard_typesById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetCard_typesById.id", id)
	card_types, err = r.card_typesRepo.GetCard_typesById(ctx, id)
	if err != nil {
		return card_types, errors.New(app.MsgRepositoryError)
	}

	return card_types, nil
}
func (r Resource)  GetCard_typesByTypeCode(ctx context.Context, typecode string) (card_types *model.Card_types, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_typesService -> GetCard_typesByTypeCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetCard_typesByTypeCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetCard_typesByTypeCode.typecode", typecode)
	card_types, err = r.card_typesRepo.GetCard_typesByTypeCode(ctx, typecode)
	if err != nil {
		return card_types, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetCard_typesByTypeCode.found", card_types != nil)
	return card_types, nil
}
func (r Resource)  DeleteCard_typesById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_typesService -> DeleteCard_typesById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteCard_typesById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteCard_typesById.id",id)

	result, err = r.card_typesRepo.DeleteCard_typesById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteCard_typesById.deleted", result)
	return result, nil
}
func (r Resource)  InsertCard_types(ctx context.Context,card_types *model.Card_types) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_typesService -> InsertCard_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertCard_types")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertCard_types.typecode", card_types.TypeCode)
	insertedId, err = r.card_typesRepo.InsertCard_types(ctx, card_types)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertCard_types.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateCard_types(ctx context.Context,card_types *model.Card_types, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_typesService -> UpdateCard_types", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateCard_types")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateCard_types.id", id)
	tracker.AddParam("service.UpdateCard_types.typecode", card_types.TypeCode)

	err = r.card_typesRepo.UpdateCard_types(ctx, card_types, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateCard_types.updated", true)

	return nil
}

