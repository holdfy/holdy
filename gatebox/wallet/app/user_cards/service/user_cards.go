package user_cardsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	user_cardsRepo "palm-pay/app/user_cards/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type User_cardsServiceIF interface {
     GetUser_cards(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_cardsById(ctx context.Context, id int64) (*model.User_cards, error)
     GetUser_cardsByCardCode(ctx context.Context, cardcode string) (*model.User_cards, error)
     InsertUser_cards(ctx context.Context, user_cards *model.User_cards) (int64, error)
     UpdateUser_cards(ctx context.Context, user_cards *model.User_cards, id int64) error
     DeleteUser_cardsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     user_cardsRepo user_cardsRepo.User_cardsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewUser_cardsService(user_cardsRepo user_cardsRepo.User_cardsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         user_cardsRepo: user_cardsRepo,
		  observability:  observabilidade.NewServiceObservability("service.user_cards"),
     }
}
func (r Resource)  GetUser_cards(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_cardsService -> GetUser_cards", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_cards.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetUser_cards.offset", offset)
	tracker.AddParam("service.GetUser_cards.limit", limit)



	itemsPage, err = r.user_cardsRepo.GetUser_cards(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.User_cards); ok {
		tracker.AddResult("service.GetUser_cards.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetUser_cardsById(ctx context.Context, id int64) (user_cards *model.User_cards, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_cardsService -> GetUser_cardsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_cardsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetUser_cardsById.id", id)
	user_cards, err = r.user_cardsRepo.GetUser_cardsById(ctx, id)
	if err != nil {
		return user_cards, errors.New(app.MsgRepositoryError)
	}

	return user_cards, nil
}
func (r Resource)  GetUser_cardsByCardCode(ctx context.Context, cardcode string) (user_cards *model.User_cards, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_cardsService -> GetUser_cardsByCardCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetUser_cardsByCardCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetUser_cardsByCardCode.cardcode", cardcode)
	user_cards, err = r.user_cardsRepo.GetUser_cardsByCardCode(ctx, cardcode)
	if err != nil {
		return user_cards, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetUser_cardsByCardCode.found", user_cards != nil)
	return user_cards, nil
}
func (r Resource)  DeleteUser_cardsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_cardsService -> DeleteUser_cardsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteUser_cardsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteUser_cardsById.id",id)

	result, err = r.user_cardsRepo.DeleteUser_cardsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteUser_cardsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertUser_cards(ctx context.Context,user_cards *model.User_cards) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_cardsService -> InsertUser_cards", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertUser_cards")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertUser_cards.cardcode", user_cards.CardCode)
	insertedId, err = r.user_cardsRepo.InsertUser_cards(ctx, user_cards)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertUser_cards.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateUser_cards(ctx context.Context,user_cards *model.User_cards, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_cardsService -> UpdateUser_cards", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateUser_cards")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateUser_cards.id", id)
	tracker.AddParam("service.UpdateUser_cards.cardcode", user_cards.CardCode)

	err = r.user_cardsRepo.UpdateUser_cards(ctx, user_cards, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateUser_cards.updated", true)

	return nil
}

