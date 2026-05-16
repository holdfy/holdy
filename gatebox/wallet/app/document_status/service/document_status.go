package document_statusSV

import (
  "context"
  "errors"

	app "palm-pay/app"
	"time"
	"palm-pay/utils/observabilidade"
	document_statusRepo "palm-pay/app/document_status/repository"
	model "palm-pay/model"
	"github.com/tungstenbyte/utils/logger"
)
 type Document_statusServiceIF interface {
     GetDocument_status(ctx context.Context, offset int64, limit int64) (model.ItemsPage, error)
     GetDocument_statusById(ctx context.Context, id int64) (*model.Document_status, error)
     GetDocument_statusByStatusCode(ctx context.Context, statuscode string) (*model.Document_status, error)
     InsertDocument_status(ctx context.Context, document_status *model.Document_status) (int64, error)
     UpdateDocument_status(ctx context.Context, document_status *model.Document_status, id int64) error
     DeleteDocument_statusById(ctx context.Context, id int64) (bool, error)
}
 type Resource struct {
     document_statusRepo document_statusRepo.Document_statusRepositoryIF
     log     logger.Logger
	  observability  *observabilidade.ServiceObservability
}
 func NewDocument_statusService(document_statusRepo document_statusRepo.Document_statusRepositoryIF, log logger.Logger) *Resource{
    return &Resource{
         log:     log,
         document_statusRepo: document_statusRepo,
		  observability:  observabilidade.NewServiceObservability("service.document_status"),
     }
}
func (r Resource)  GetDocument_status(ctx context.Context, offset int64, limit int64) (itemsPage model.ItemsPage, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_statusService -> GetDocument_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetDocument_status.get_list")
	defer tracker.Finish(&err)

	tracker.AddParam("service.GetDocument_status.offset", offset)
	tracker.AddParam("service.GetDocument_status.limit", limit)



	itemsPage, err = r.document_statusRepo.GetDocument_status(ctx, offset, limit)
	if err != nil {
		return itemsPage, errors.New(app.MsgRepositoryError)
	}

	if items, ok := itemsPage.Items.([]model.Document_status); ok {
		tracker.AddResult("service.GetDocument_status.count", len(items))
	}

	return itemsPage, nil
}
func (r Resource)  GetDocument_statusById(ctx context.Context, id int64) (document_status *model.Document_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_statusService -> GetDocument_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.GetDocument_statusById")
	defer tracker.Finish(&err)
   tracker.AddParam("service.GetDocument_statusById.id", id)
	document_status, err = r.document_statusRepo.GetDocument_statusById(ctx, id)
	if err != nil {
		return document_status, errors.New(app.MsgRepositoryError)
	}

	return document_status, nil
}
func (r Resource)  GetDocument_statusByStatusCode(ctx context.Context, statuscode string) (document_status *model.Document_status, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_statusService -> GetDocument_statusByStatusCode", &startedAt)

   tracker := r.observability.Track(ctx, "service.GetDocument_statusByStatusCode")
   defer tracker.Finish(&err)

   tracker.AddParam("service.GetDocument_statusByStatusCode.statuscode", statuscode)
	document_status, err = r.document_statusRepo.GetDocument_statusByStatusCode(ctx, statuscode)
	if err != nil {
		return document_status, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.GetDocument_statusByStatusCode.found", document_status != nil)
	return document_status, nil
}
func (r Resource)  DeleteDocument_statusById(ctx context.Context, id int64) (result bool, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_statusService -> DeleteDocument_statusById", &startedAt)

	tracker := r.observability.Track(ctx, "service.DeleteDocument_statusById.delete")
	defer tracker.Finish(&err)

	tracker.AddParam("service.DeleteDocument_statusById.id",id)

	result, err = r.document_statusRepo.DeleteDocument_statusById(ctx, id)

	if err != nil {
		return result, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.DeleteDocument_statusById.deleted", result)
	return result, nil
}
func (r Resource)  InsertDocument_status(ctx context.Context,document_status *model.Document_status) (insertedId int64, err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_statusService -> InsertDocument_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.InsertDocument_status")
	defer tracker.Finish(&err)

	tracker.AddParam("service.InsertDocument_status.statuscode", document_status.StatusCode)
	insertedId, err = r.document_statusRepo.InsertDocument_status(ctx, document_status)
	if err != nil {
		return 0, errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.InsertDocument_status.inserted_id", insertedId)
	return insertedId, nil
}
func (r Resource)  UpdateDocument_status(ctx context.Context,document_status *model.Document_status, id int64) (err error) {
	startedAt := time.Now()
	defer r.log.Chronometer("Document_statusService -> UpdateDocument_status", &startedAt)

	tracker := r.observability.Track(ctx, "service.UpdateDocument_status")
	defer tracker.Finish(&err)
	tracker.AddParam("service.UpdateDocument_status.id", id)
	tracker.AddParam("service.UpdateDocument_status.statuscode", document_status.StatusCode)

	err = r.document_statusRepo.UpdateDocument_status(ctx, document_status, id)
	if err != nil {
		return errors.New(app.MsgRepositoryError)
	}

	tracker.AddResult("service.UpdateDocument_status.updated", true)

	return nil
}

