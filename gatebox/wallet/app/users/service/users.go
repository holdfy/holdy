package usersSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	usersRepo "palm-pay/app/users/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type UsersServiceIF interface {
     GetUsers(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUsersById(ctx context.Context, id int64) (*model.Users, error)
     GetUsersByUserCode(ctx context.Context, usercode string) (*model.Users, error)
     InsertUsers(ctx context.Context, users *model.Users) (int64, error)
     UpdateUsers(ctx context.Context, users *model.Users, id int64) error
     DeleteUsersById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     usersRepo usersRepo.UsersRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewUsersService(usersRepo usersRepo.UsersRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         usersRepo: usersRepo,
		  observability:  observabilidade.NewServiceObservability("service.users"),
     }
}
func (r Resource)  GetUsers(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("UsersService -> GetUsers", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUsers.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetUsers.offset", offset)
	tracker.AddParam("service.GetUsers.limit", limit)



	itemsPage, err = r.usersRepo.GetUsers(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Users); ok {
		tracker.AddResult("service.GetUsers.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetUsersById(ctx context.Context, id int64) (users *model.Users, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("UsersService -> GetUsersById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUsersById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetUsersById.id", id)
	users, err = r.usersRepo.GetUsersById(ctx, id)
	if err != nil {
		return users, errors.New(app.MsgRepositoryError)
	}

	return users, nil
}
func (r Resource)  GetUsersByUserCode(ctx context.Context, usercode string) (users *model.Users, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("UsersService -> GetUsersByUserCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetUsersByUserCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetUsersByUserCode.usercode", usercode)
	users, err = r.usersRepo.GetUsersByUserCode(ctx, usercode)
	if err != nil {
		return users, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetUsersByUserCode.found", users != nil)
	return users, nil
}
func (r Resource)  DeleteUsersById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("UsersService -> DeleteUsersById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteUsersById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteUsersById.id",id)

	result, err = r.usersRepo.DeleteUsersById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteUsersById.deleted", result)
	return result, nil
}
func (r Resource)  InsertUsers(ctx context.Context,users *model.Users) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("UsersService -> InsertUsers", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertUsers")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertUsers.usercode", users.UserCode)
	insertedId, err = r.usersRepo.InsertUsers(ctx, users)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertUsers.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateUsers(ctx context.Context,users *model.Users, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("UsersService -> UpdateUsers", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateUsers")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateUsers.id", id)
	tracker.AddParam("service.UpdateUsers.usercode", users.UserCode)

	err = r.usersRepo.UpdateUsers(ctx, users, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateUsers.updated", true)

	return nil
}

