package user_documentsSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	user_documentsRepo "palm-pay/app/user_documents/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type User_documentsServiceIF interface {
     GetUser_documents(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetUser_documentsById(ctx context.Context, id int64) (*model.User_documents, error)
     GetUser_documentsByDocumentCode(ctx context.Context, documentcode string) (*model.User_documents, error)
     InsertUser_documents(ctx context.Context, user_documents *model.User_documents) (int64, error)
     UpdateUser_documents(ctx context.Context, user_documents *model.User_documents, id int64) error
     DeleteUser_documentsById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     user_documentsRepo user_documentsRepo.User_documentsRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewUser_documentsService(user_documentsRepo user_documentsRepo.User_documentsRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         user_documentsRepo: user_documentsRepo,
		  observability:  observabilidade.NewServiceObservability("service.user_documents"),
     }
}
func (r Resource)  GetUser_documents(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_documentsService -> GetUser_documents", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_documents.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetUser_documents.offset", offset)
	tracker.AddParam("service.GetUser_documents.limit", limit)



	itemsPage, err = r.user_documentsRepo.GetUser_documents(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.User_documents); ok {
		tracker.AddResult("service.GetUser_documents.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetUser_documentsById(ctx context.Context, id int64) (user_documents *model.User_documents, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_documentsService -> GetUser_documentsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetUser_documentsById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetUser_documentsById.id", id)
	user_documents, err = r.user_documentsRepo.GetUser_documentsById(ctx, id)
	if err != nil {
		return user_documents, errors.New(app.MsgRepositoryError)
	}

	return user_documents, nil
}
func (r Resource)  GetUser_documentsByDocumentCode(ctx context.Context, documentcode string) (user_documents *model.User_documents, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_documentsService -> GetUser_documentsByDocumentCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetUser_documentsByDocumentCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetUser_documentsByDocumentCode.documentcode", documentcode)
	user_documents, err = r.user_documentsRepo.GetUser_documentsByDocumentCode(ctx, documentcode)
	if err != nil {
		return user_documents, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetUser_documentsByDocumentCode.found", user_documents != nil)
	return user_documents, nil
}
func (r Resource)  DeleteUser_documentsById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_documentsService -> DeleteUser_documentsById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteUser_documentsById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteUser_documentsById.id",id)

	result, err = r.user_documentsRepo.DeleteUser_documentsById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteUser_documentsById.deleted", result)
	return result, nil
}
func (r Resource)  InsertUser_documents(ctx context.Context,user_documents *model.User_documents) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_documentsService -> InsertUser_documents", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertUser_documents")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertUser_documents.documentcode", user_documents.DocumentCode)
	insertedId, err = r.user_documentsRepo.InsertUser_documents(ctx, user_documents)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertUser_documents.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateUser_documents(ctx context.Context,user_documents *model.User_documents, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("User_documentsService -> UpdateUser_documents", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateUser_documents")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateUser_documents.id", id)
	tracker.AddParam("service.UpdateUser_documents.documentcode", user_documents.DocumentCode)

	err = r.user_documentsRepo.UpdateUser_documents(ctx, user_documents, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateUser_documents.updated", true)

	return nil
}

