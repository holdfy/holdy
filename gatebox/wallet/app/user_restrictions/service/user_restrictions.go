package user_restrictionsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	user_restrictionsRepo "palm-pay/app/user_restrictions/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type User_restrictionsServiceIF interface {
     GetUser_restrictions(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_restrictionsById(ctx context.Context, id int64) (*model.User_restrictions, error)
     GetUser_restrictionsByRestrictionCode(ctx context.Context, restrictioncode string) (*model.User_restrictions, error)
     InsertUser_restrictions(ctx context.Context, user_restrictions *model.User_restrictions) (int64, error)
     UpdateUser_restrictions(ctx context.Context, user_restrictions *model.User_restrictions, id int64) error
     DeleteUser_restrictionsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     user_restrictionsRepo user_restrictionsRepo.User_restrictionsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewUser_restrictionsService(user_restrictionsRepo user_restrictionsRepo.User_restrictionsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         user_restrictionsRepo: user_restrictionsRepo,
		  observability:  observabilidade.NewServiceObservability("service.user_restrictions"),
     }
}
func (r Resource)  GetUser_restrictions(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_restrictionsService -> GetUser_restrictions", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_restrictions.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetUser_restrictions.offset", offset)
	tracker.AddParam("service.GetUser_restrictions.limit", limit)



	itemsPage, err = r.user_restrictionsRepo.GetUser_restrictions(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.User_restrictions); ok {
		tracker.AddResult("service.GetUser_restrictions.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetUser_restrictionsById(ctx context.Context, id int64) (user_restrictions *model.User_restrictions, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_restrictionsService -> GetUser_restrictionsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_restrictionsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetUser_restrictionsById.id", id)
	user_restrictions, err = r.user_restrictionsRepo.GetUser_restrictionsById(ctx, id)
	if err != nil {
		return user_restrictions, errors.New(app.MsgRepositoryError)
	}

	return user_restrictions, nil
}
func (r Resource)  GetUser_restrictionsByRestrictionCode(ctx context.Context, restrictioncode string) (user_restrictions *model.User_restrictions, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_restrictionsService -> GetUser_restrictionsByRestrictionCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetUser_restrictionsByRestrictionCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetUser_restrictionsByRestrictionCode.restrictioncode", restrictioncode)
	user_restrictions, err = r.user_restrictionsRepo.GetUser_restrictionsByRestrictionCode(ctx, restrictioncode)
	if err != nil {
		return user_restrictions, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetUser_restrictionsByRestrictionCode.found", user_restrictions != nil)
	return user_restrictions, nil
}
func (r Resource)  DeleteUser_restrictionsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_restrictionsService -> DeleteUser_restrictionsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteUser_restrictionsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteUser_restrictionsById.id",id)

	result, err = r.user_restrictionsRepo.DeleteUser_restrictionsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteUser_restrictionsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertUser_restrictions(ctx context.Context,user_restrictions *model.User_restrictions) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_restrictionsService -> InsertUser_restrictions", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertUser_restrictions")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertUser_restrictions.restrictioncode", user_restrictions.RestrictionCode)
	insertedId, err = r.user_restrictionsRepo.InsertUser_restrictions(ctx, user_restrictions)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertUser_restrictions.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateUser_restrictions(ctx context.Context,user_restrictions *model.User_restrictions, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_restrictionsService -> UpdateUser_restrictions", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateUser_restrictions")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateUser_restrictions.id", id)
	tracker.AddParam("service.UpdateUser_restrictions.restrictioncode", user_restrictions.RestrictionCode)

	err = r.user_restrictionsRepo.UpdateUser_restrictions(ctx, user_restrictions, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateUser_restrictions.updated", true)

	return nil
}

