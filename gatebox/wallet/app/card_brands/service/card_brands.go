package card_brandsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	card_brandsRepo "palm-pay/app/card_brands/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Card_brandsServiceIF interface {
     GetCard_brands(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetCard_brandsById(ctx context.Context, id int64) (*model.Card_brands, error)
     GetCard_brandsByBrandCode(ctx context.Context, brandcode string) (*model.Card_brands, error)
     InsertCard_brands(ctx context.Context, card_brands *model.Card_brands) (int64, error)
     UpdateCard_brands(ctx context.Context, card_brands *model.Card_brands, id int64) error
     DeleteCard_brandsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     card_brandsRepo card_brandsRepo.Card_brandsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewCard_brandsService(card_brandsRepo card_brandsRepo.Card_brandsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         card_brandsRepo: card_brandsRepo,
		  observability:  observabilidade.NewServiceObservability("service.card_brands"),
     }
}
func (r Resource)  GetCard_brands(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_brandsService -> GetCard_brands", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetCard_brands.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetCard_brands.offset", offset)
	tracker.AddParam("service.GetCard_brands.limit", limit)



	itemsPage, err = r.card_brandsRepo.GetCard_brands(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Card_brands); ok {
		tracker.AddResult("service.GetCard_brands.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetCard_brandsById(ctx context.Context, id int64) (card_brands *model.Card_brands, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_brandsService -> GetCard_brandsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetCard_brandsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetCard_brandsById.id", id)
	card_brands, err = r.card_brandsRepo.GetCard_brandsById(ctx, id)
	if err != nil {
		return card_brands, errors.New(app.MsgRepositoryError)
	}

	return card_brands, nil
}
func (r Resource)  GetCard_brandsByBrandCode(ctx context.Context, brandcode string) (card_brands *model.Card_brands, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_brandsService -> GetCard_brandsByBrandCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetCard_brandsByBrandCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetCard_brandsByBrandCode.brandcode", brandcode)
	card_brands, err = r.card_brandsRepo.GetCard_brandsByBrandCode(ctx, brandcode)
	if err != nil {
		return card_brands, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetCard_brandsByBrandCode.found", card_brands != nil)
	return card_brands, nil
}
func (r Resource)  DeleteCard_brandsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_brandsService -> DeleteCard_brandsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteCard_brandsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteCard_brandsById.id",id)

	result, err = r.card_brandsRepo.DeleteCard_brandsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteCard_brandsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertCard_brands(ctx context.Context,card_brands *model.Card_brands) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_brandsService -> InsertCard_brands", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertCard_brands")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertCard_brands.brandcode", card_brands.BrandCode)
	insertedId, err = r.card_brandsRepo.InsertCard_brands(ctx, card_brands)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertCard_brands.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateCard_brands(ctx context.Context,card_brands *model.Card_brands, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Card_brandsService -> UpdateCard_brands", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateCard_brands")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateCard_brands.id", id)
	tracker.AddParam("service.UpdateCard_brands.brandcode", card_brands.BrandCode)

	err = r.card_brandsRepo.UpdateCard_brands(ctx, card_brands, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateCard_brands.updated", true)

	return nil
}

