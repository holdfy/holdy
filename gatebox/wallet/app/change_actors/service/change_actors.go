package change_actorsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	change_actorsRepo "palm-pay/app/change_actors/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Change_actorsServiceIF interface {
     GetChange_actors(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetChange_actorsById(ctx context.Context, id int64) (*model.Change_actors, error)
     GetChange_actorsByActorCode(ctx context.Context, actorcode string) (*model.Change_actors, error)
     InsertChange_actors(ctx context.Context, change_actors *model.Change_actors) (int64, error)
     UpdateChange_actors(ctx context.Context, change_actors *model.Change_actors, id int64) error
     DeleteChange_actorsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     change_actorsRepo change_actorsRepo.Change_actorsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewChange_actorsService(change_actorsRepo change_actorsRepo.Change_actorsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         change_actorsRepo: change_actorsRepo,
		  observability:  observabilidade.NewServiceObservability("service.change_actors"),
     }
}
func (r Resource)  GetChange_actors(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Change_actorsService -> GetChange_actors", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetChange_actors.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetChange_actors.offset", offset)
	tracker.AddParam("service.GetChange_actors.limit", limit)



	itemsPage, err = r.change_actorsRepo.GetChange_actors(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Change_actors); ok {
		tracker.AddResult("service.GetChange_actors.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetChange_actorsById(ctx context.Context, id int64) (change_actors *model.Change_actors, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Change_actorsService -> GetChange_actorsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetChange_actorsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetChange_actorsById.id", id)
	change_actors, err = r.change_actorsRepo.GetChange_actorsById(ctx, id)
	if err != nil {
		return change_actors, errors.New(app.MsgRepositoryError)
	}

	return change_actors, nil
}
func (r Resource)  GetChange_actorsByActorCode(ctx context.Context, actorcode string) (change_actors *model.Change_actors, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Change_actorsService -> GetChange_actorsByActorCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetChange_actorsByActorCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetChange_actorsByActorCode.actorcode", actorcode)
	change_actors, err = r.change_actorsRepo.GetChange_actorsByActorCode(ctx, actorcode)
	if err != nil {
		return change_actors, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetChange_actorsByActorCode.found", change_actors != nil)
	return change_actors, nil
}
func (r Resource)  DeleteChange_actorsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Change_actorsService -> DeleteChange_actorsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteChange_actorsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteChange_actorsById.id",id)

	result, err = r.change_actorsRepo.DeleteChange_actorsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteChange_actorsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertChange_actors(ctx context.Context,change_actors *model.Change_actors) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Change_actorsService -> InsertChange_actors", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertChange_actors")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertChange_actors.actorcode", change_actors.ActorCode)
	insertedId, err = r.change_actorsRepo.InsertChange_actors(ctx, change_actors)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertChange_actors.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateChange_actors(ctx context.Context,change_actors *model.Change_actors, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Change_actorsService -> UpdateChange_actors", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateChange_actors")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateChange_actors.id", id)
	tracker.AddParam("service.UpdateChange_actors.actorcode", change_actors.ActorCode)

	err = r.change_actorsRepo.UpdateChange_actors(ctx, change_actors, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateChange_actors.updated", true)

	return nil
}

