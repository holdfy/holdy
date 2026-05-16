package user_statusSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	user_statusRepo "palm-pay/app/user_status/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type User_statusServiceIF interface {
     GetUser_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_statusById(ctx context.Context, id int64) (*model.User_status, error)
     GetUser_statusByStatusCode(ctx context.Context, statuscode string) (*model.User_status, error)
     InsertUser_status(ctx context.Context, user_status *model.User_status) (int64, error)
     UpdateUser_status(ctx context.Context, user_status *model.User_status, id int64) error
     DeleteUser_statusById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     user_statusRepo user_statusRepo.User_statusRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewUser_statusService(user_statusRepo user_statusRepo.User_statusRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         user_statusRepo: user_statusRepo,
		  observability:  observabilidade.NewServiceObservability("service.user_status"),
     }
}
func (r Resource)  GetUser_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_statusService -> GetUser_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_status.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetUser_status.offset", offset)
	tracker.AddParam("service.GetUser_status.limit", limit)



	itemsPage, err = r.user_statusRepo.GetUser_status(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.User_status); ok {
		tracker.AddResult("service.GetUser_status.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetUser_statusById(ctx context.Context, id int64) (user_status *model.User_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_statusService -> GetUser_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_statusById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetUser_statusById.id", id)
	user_status, err = r.user_statusRepo.GetUser_statusById(ctx, id)
	if err != nil {
		return user_status, errors.New(app.MsgRepositoryError)
	}

	return user_status, nil
}
func (r Resource)  GetUser_statusByStatusCode(ctx context.Context, statuscode string) (user_status *model.User_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_statusService -> GetUser_statusByStatusCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetUser_statusByStatusCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetUser_statusByStatusCode.statuscode", statuscode)
	user_status, err = r.user_statusRepo.GetUser_statusByStatusCode(ctx, statuscode)
	if err != nil {
		return user_status, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetUser_statusByStatusCode.found", user_status != nil)
	return user_status, nil
}
func (r Resource)  DeleteUser_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_statusService -> DeleteUser_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteUser_statusById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteUser_statusById.id",id)

	result, err = r.user_statusRepo.DeleteUser_statusById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteUser_statusById.deleted", result)
	return result, nil
}
func (r Resource)  InsertUser_status(ctx context.Context,user_status *model.User_status) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_statusService -> InsertUser_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertUser_status")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertUser_status.statuscode", user_status.StatusCode)
	insertedId, err = r.user_statusRepo.InsertUser_status(ctx, user_status)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertUser_status.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateUser_status(ctx context.Context,user_status *model.User_status, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_statusService -> UpdateUser_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateUser_status")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateUser_status.id", id)
	tracker.AddParam("service.UpdateUser_status.statuscode", user_status.StatusCode)

	err = r.user_statusRepo.UpdateUser_status(ctx, user_status, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateUser_status.updated", true)

	return nil
}

