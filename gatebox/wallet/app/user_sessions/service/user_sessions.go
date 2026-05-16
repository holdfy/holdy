package user_sessionsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	user_sessionsRepo "palm-pay/app/user_sessions/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type User_sessionsServiceIF interface {
     GetUser_sessions(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_sessionsById(ctx context.Context, id int64) (*model.User_sessions, error)
     GetUser_sessionsBySessionCode(ctx context.Context, sessioncode string) (*model.User_sessions, error)
     InsertUser_sessions(ctx context.Context, user_sessions *model.User_sessions) (int64, error)
     UpdateUser_sessions(ctx context.Context, user_sessions *model.User_sessions, id int64) error
     DeleteUser_sessionsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     user_sessionsRepo user_sessionsRepo.User_sessionsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewUser_sessionsService(user_sessionsRepo user_sessionsRepo.User_sessionsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         user_sessionsRepo: user_sessionsRepo,
		  observability:  observabilidade.NewServiceObservability("service.user_sessions"),
     }
}
func (r Resource)  GetUser_sessions(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_sessionsService -> GetUser_sessions", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_sessions.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetUser_sessions.offset", offset)
	tracker.AddParam("service.GetUser_sessions.limit", limit)



	itemsPage, err = r.user_sessionsRepo.GetUser_sessions(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.User_sessions); ok {
		tracker.AddResult("service.GetUser_sessions.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetUser_sessionsById(ctx context.Context, id int64) (user_sessions *model.User_sessions, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_sessionsService -> GetUser_sessionsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_sessionsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetUser_sessionsById.id", id)
	user_sessions, err = r.user_sessionsRepo.GetUser_sessionsById(ctx, id)
	if err != nil {
		return user_sessions, errors.New(app.MsgRepositoryError)
	}

	return user_sessions, nil
}
func (r Resource)  GetUser_sessionsBySessionCode(ctx context.Context, sessioncode string) (user_sessions *model.User_sessions, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_sessionsService -> GetUser_sessionsBySessionCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetUser_sessionsBySessionCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetUser_sessionsBySessionCode.sessioncode", sessioncode)
	user_sessions, err = r.user_sessionsRepo.GetUser_sessionsBySessionCode(ctx, sessioncode)
	if err != nil {
		return user_sessions, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetUser_sessionsBySessionCode.found", user_sessions != nil)
	return user_sessions, nil
}
func (r Resource)  DeleteUser_sessionsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_sessionsService -> DeleteUser_sessionsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteUser_sessionsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteUser_sessionsById.id",id)

	result, err = r.user_sessionsRepo.DeleteUser_sessionsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteUser_sessionsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertUser_sessions(ctx context.Context,user_sessions *model.User_sessions) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_sessionsService -> InsertUser_sessions", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertUser_sessions")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertUser_sessions.sessioncode", user_sessions.SessionCode)
	insertedId, err = r.user_sessionsRepo.InsertUser_sessions(ctx, user_sessions)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertUser_sessions.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateUser_sessions(ctx context.Context,user_sessions *model.User_sessions, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_sessionsService -> UpdateUser_sessions", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateUser_sessions")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateUser_sessions.id", id)
	tracker.AddParam("service.UpdateUser_sessions.sessioncode", user_sessions.SessionCode)

	err = r.user_sessionsRepo.UpdateUser_sessions(ctx, user_sessions, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateUser_sessions.updated", true)

	return nil
}

